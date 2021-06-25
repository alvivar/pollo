# pollo

Simple Pub Sub with Rust. Using polling from smol.

Next step is to make it multi-thread.

I'm prototyping how to handle subscriptions to improve
[Bite](https://github.com/alvivar/bite).

# The idea

To subscribe, send a message starting with **+**, the first word after **+**
will be used as the channel to be subscribed.

    + somechannel

If you send more words, those will be considered a first message.

    + somechannel Subscribed and a first message!

To send a message without subscribing, use **:**

    : somechannel Some message to all somechannel subscribers.

To unsubscribe use **-**

    - somechannel

It's also possible to send a last message while unsubscribing:

    - somechannel And a last message.

# Try it

With Rust installed, just run:

    cargo run

In unix consoles you could use **nc** as client:

    $ nc 127.0.0.1 1984
