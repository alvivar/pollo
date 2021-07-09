# pollo

Simple multi-thread [Pub
Sub](https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern) server in
Rust.

I wanted to learn how to use [**Polling**](https://github.com/smol-rs/polling)
from [**smol**](https://github.com/smol-rs/smol) with a thread pool.

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

But any TcpStream stream from your favorite language will do.

## Warning

I learned a lot, but the architecture is flawed.

If you want this same API without problems check out the improved
[atomsub](https://github.com/alvivar/atomsub).

Right now, reading and writing is handled independently, so you can read and
write at the same time safely. But It turns out that while the thread pool has
the connection queued, that connection can't be used for the same operation
until that job is done. So, on super fast events (milliseconds) you can loose
information.

Like trying to write super fast twice in a row, or reading twice super fast from
a client.
