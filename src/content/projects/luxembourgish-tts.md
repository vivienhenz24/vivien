---
title: "Luxembourgish text-to-speech"
description: "A text-to-speech system for the Luxembourgish language, making digital content accessible to Luxembourgish speakers."
url: "https://neiom.io"
date: "2025-10-15"
---

I may be a complete noob compared to actual ML experts bu tafter having built a fair amount of transformers, I'm inclined to say that the direction in which AI is heading is completely wrong. That's just my opinion (as a noob), but I feel like there is a general sense in the field that we may be digging in the wrong direction.

I feel like most people will agree that transformer-based models don't learn, they just pretty overfit the entire english language. We got away with it by scaling compute and using insane amounts of data, but now we're running out of the latter. The consequence is that languages like Luxembourgish, which fall out of the data distribution, still aren't supported by the most advanced TTS and ASR models.

So, since I'm not held back by my X $billion investments in compute infrastructure, I want to build a model which is capable of handling out-of-scope data by shifting the emphasis away from scaling pre-training data and towards continuous learning. I'm starting with Luxembourgish because that's what I know best!

For my first iteration all I did was fork the open source fish-speech TTS model and wrote my own training loop to fine tune it on 32000 recorded Luxembourgish samples. It took 5h on an RTX5090 (~10$). 

[Visit neiom.io →](https://neiom.io)
