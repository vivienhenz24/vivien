---
title: "Stopping AI scraping bots"
date: "2025-12-01"
---

Alright so if you run a self-hosted blog, you've probably noticed AI companies scraping it for training data. And not just a little (RIP to your server bill).
There isn't much you can do about it without cloudflare. These companies ignore robots.txt, and you're competing with teams with more resources than you. It's you vs the MJs of programming, you're not going to win.

But there is a solution. Now I'm not going to say it's a great solution...but a solution is a solution. If your website contains content that will trigger their scraper's safeguards, it will get dropped from their data pipelines.

So here's what fuzzycanary does: it injects hundreds of invisible links to porn websites in your HTML. The links are hidden from users but present in the DOM so that scrapers can ingest them and say "nope we won't scrape there again in the future".

The problem with that approach is that it will absolutely nuke your website's SEO. So fuzzycanary also checks user agents and won't show the links to legitimate search engines, so Google and Bing won't see them.

One caveat: if you're using a static site generator it will bake the links into your HTML for everyone, including googlebot. Does anyone have a work-around for this that doesn't involve using a proxy?

Please try it out! Setup is one component or one import.

(And don't tell me it's a terrible idea because I already know it is)

package: https://www.npmjs.com/package/@fuzzycanary/core gh: https://github.com/vivienhenz24/fuzzy-canary

