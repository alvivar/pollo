# pollo

Simple Pub Sub with Rust. The only dependency is
[**Polling**](https://github.com/smol-rs/polling) from
[**smol**](https://github.com/smol-rs/smol) and the standard library.

Next step is to make it multi-thread, later on websockets.

I'm prototyping how to handle clients and subscriptions to improve
[Bite](https://github.com/alvivar/bite), a bigger more complex _key value_
server I use in multiplayer experiments.

## How it works

To subscribe, send a message starting with **+**, the first word after **+**
will be used as the channel to be subscribed.

    $ +somechannel

If you send more words, those will be considered a first message.

    $ +somechannel Subscribed and a first message!

To send a message without subscribing, use **:**

    $ :somechannel Some message to all somechannel subscribers.

To unsubscribe use **-**

    $ -somechannel

It's also possible to send a last message while unsubscribing:

    $ -somechannel And a last message.

When you are subscribed to a channel, you also receive the key in the first word
of the message. This way clients can react properly.

    $ somechannel Some message to this channel.

## Try it

With Rust installed, just run:

    cargo run

In unix consoles you can connect with
[**nc**](https://en.wikipedia.org/wiki/Netcat) as client:

    $ nc 127.0.0.1 1984
