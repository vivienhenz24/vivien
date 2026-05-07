---
title: "A Fault-Tolerant Pub/Sub System Built on Paxos"
description: "A fault-tolerant publish-subscribe system built on top of pancy, a multi-Paxos key-value store. Routes every publish through the Paxos log before notifying subscribers, giving ordering guarantees for free."
date: "2026-05-01"
---

*CS 2620 — Spring 2026*

## Introduction

Publish-subscribe (pub/sub) is a way of communication in which producers emit messages on "topics" without knowledge of who is listening, and consumers receive all messages on topics they have subscribed to.

Imagine you want to order pizza and want to know which of your friends would also like to order. Instead of texting each of your friends individually "yo do you want pizza?", you could create a group chat dedicated to pizza orders and ask everyone in the group if they want pizza once.

That's an example of a pub/sub system. The group chat is the topic, and the messages are the pizza orders.

This decoupling is super efficient but of course the catch is that if the broker (the groupchat for example) that routes messages crashes then messages can be lost and subscribers won't get notified.

The fix is to replicate the broker so no single crash can lose messages. Consensus protocols such as Paxos are a super good fit here since Paxos keeps a fault-tolerant, totally-ordered log of every operation across replicas. In our case, the publish is just a log entry, and a notification is what happens once that entry commits. Ordering comes for free from the consensus guarantees, with no extra coordination needed.

This writeup describes the pub/sub system I built on top of pancy, which is the multi-Paxos key-value store we worked on in pset3. The main idea was to add 4 new message types to the pancy protocol: `subscribe`, `unsubscribe`, `publish`, and `notify`. Then I added subscription tracking and notification fanout directly in the replica layer. Subscribers recover from leader failure by re-subscribing through a sequence number, which ensures no notifications are missed across failures.

## Related Work

I've had the pleasure of working with Apache Kafka during my last internship, which is why I chose this topic for my project. Kafka is a fault-tolerant pub/sub system widely used in industry — it replicates a partitioned log across brokers to provide ordered delivery. My system has the same core idea: route every publish through an ordered log before notifying subscribers using Paxos.

Google's Pub/Sub service separates storage from serving and uses a distributed queue with at-least-once delivery guarantees. That's different from my system, which targets at-most-once delivery per subscriber session — more appropriate for real-time notifications and simpler to implement on top of Paxos.

## Design

### Message Protocol

I added four new message types to the pancy message layer:

- **`subscribe` / `unsubscribe`**: A client sends a `subscribe_request` with a topic name and an optional `last_seq` — the Paxos slot number of the last notification it processed. The replica records the subscription and confirms. Unsubscribe removes the client from the topic.

- **`publish`**: A client sends a `publish_request` with a topic and payload. The replica does not notify subscribers immediately — it first proposes the publish through Paxos. Only once the slot commits does it send a `publish_response` to the publisher and emit a `notify_message` to all subscribers on that topic.

- **`notify`**: A server-push message, not tied to any request. Carries the topic name, payload, and the committed Paxos slot number as `seq_num`.

The key design decision is routing publishes through Paxos before notifying. That means delivery ordering is guaranteed by consensus — subscribers always see notifications in the same order the publishes committed, regardless of which replica they're connected to.

### Replica State

Each Paxos replica tracks two new pieces of state:

```cpp
std::map<std::string, std::vector<uint64_t>> subscriptions_;
std::vector<pancy::notify_message>           pending_notifies_;
```

`subscriptions_` maps each topic to the list of subscriber serials currently enrolled. When a publish commits, the replica appends one `notify_message` per subscriber to `pending_notifies_`. These are async-flushed after each Paxos decision.

The `seq_num` in each notification is the Paxos slot number of the committed publish. Since slot numbers are globally increasing, a subscriber's `seq_num` values are always increasing even when leaders fail and there's a resubscription.

### Client Model

The client model runs two kinds of coroutines.

**Publisher.** Periodically picks a random topic and payload and sends a `publish_request` to its current leader. On timeout it retries by rotating to a new random replica.

**Subscriber.** Sends a `subscribe_request` with its `last_seq` and retries until confirmed. Then enters a notification loop with two correctness rules:

1. *Stale-notification suppression.* Any notification with `seq_num` ≤ `last_seq` is dropped. This handles duplicates that arrive after re-subscribing to a new leader.

2. *Timeout-based failure detection.* If no notification arrives within the timeout window, the subscriber assumes the leader has failed and picks a new replica at random. It re-subscribes with its current `last_seq` so it picks up exactly where it left off.

## Implementation

### Message Types in `pancy_msgs.hh`

The new message types were initially defined in a separate `pubsub_msgs.hh` file, but this caused a circular import where each header needed to include the other. The simplest fix was adding the new types directly into `pancy_msgs.hh`.

### Coroutine Initialization Order

Coroutines in cotamer resume immediately at their first suspension point, so launching a coroutine before its state entry exists in the vector causes a segfault. The fix is a two-pass init in `start()`: first insert all state entries, then launch all coroutines.

### Buffered Notification Delivery

Notifications are buffered rather than sent immediately when a publish commits. A separate coroutine, `drain_notifies()`, flushes the buffer after each Paxos decision. This is needed because sending a notification requires `co_await`, which can't be used inside a commit callback that is itself already being awaited.

### Sequence Number Gaps Across Topics

Paxos slot numbers are global, so a subscriber on topic `alpha` will see gaps in its `seq_num` values whenever a `beta` publish occupies an intermediate slot. The correctness check was updated to only require that each new `seq_num` is greater than the previous one on the same topic rather than strictly consecutive — gaps are expected.

## Evaluation

I evaluated the system using cotamer's simulator. Each run simulates 100 seconds of operation with 2 publishers, 4 subscribers (2 per topic), and 2 topics. Three failure events are injected per run: a leader crash at t=20s (lasting 20s), a follower crash at t=45s (lasting 15s), and a network partition at t=60s (lasting 20s). Results are across 20 random seeds for both 3-replica and 5-replica configurations.

### Correctness

Zero ordering violations detected.

| Metric | 3R Min | 3R Max | 5R Min | 5R Max |
|---|---|---|---|---|
| Publishes completed | 88 | 95 | 87 | 95 |
| Notifications received | 166 | 198 | 171 | 198 |
| Ordering violations | 0 | 0 | 0 | 0 |

### Performance

| Configuration | Avg publishes/s | Avg notifications/s |
|---|---|---|
| 3 replicas (with failures) | 0.91 | 1.81 |
| 5 replicas (with failures) | 0.91 | 1.85 |

Throughput is similar across replica counts because the bottleneck is publisher pacing (each publisher waits ~500ms between requests), not consensus latency. Numbers are lower than the theoretical maximum because failure windows take up a significant portion of each run, during which publishes time out and retry instead of committing.

## Conclusion

I built a fault-tolerant pub/sub system on top of multi-Paxos. The core idea was simple: route every publish through the Paxos log before notifying subscribers, and ordering comes for free. Subscribers recovered from leader failure by re-subscribing with a `last_seq` cursor, so no notifications were missed across failovers.

Two directions for future work: first, subscription state currently lives only in the leader's memory and is not replicated — a new leader has to wait for clients to re-subscribe before it can resume emits, and replicating subscriptions through Paxos would fix this. Second, the system could be extended with topic filters and content-based routing, which are standard features in production pub/sub systems like Kafka.

---

*I used AI to format the tables in this writeup.*
