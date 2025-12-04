---
title: "Luxembourgish text-to-speech"
description: "A text-to-speech system for the Luxembourgish language, making digital content accessible to Luxembourgish speakers."
url: "https://neiom.io"
date: "2025-10-15"
---

I wanted to use a Luxembourgish text to speech model for some trolling purposes, so I went on Elevenlabs and used theirs...only to find out that it just doesnt work. They say they support Luxembourgish, but I think they don't have anyone at the office that speaks Luxembourgish that can tell them it's complete gibberish. 

I thought I could do a better job than that so for my first iteration of a Luxembourgish tts model all I did was fork the open source fish-speech TTS inference pipeline and wrote my own training loop to fine tune it on 32000 recorded Luxembourgish samples. It took 5h on an RTX5090 (~10$). 

[Visit neiom.io →](https://neiom.io)

