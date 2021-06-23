# pollo

Simple multi-thread Pub Sub with Rust. Using polling from smol.

_Work in progress!_

# The idea

To subscribe, send a message starting with **+**, the first word after **+** will be
used as the channel to be subscribed.

    + somechannel

If you send more words, those will be considered a first message.

    + somechannel Subscribed and a first message!

To send a message without subscribing, let's use **:**

    : somechannel Some message to all somechannel subscribers.

To unsubscribe makes sense to use **-**

    - somechannel

It's also possible to send a last message while unsubscribing:

    - somechannel And a last message.
