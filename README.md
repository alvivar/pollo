# pollo

Ok, this is a simple multi-thread Pub Sub while learning to use polling from
smol. I'm not ready to get into async yet, so, low level instead.

_Work in progress!_

# The idea

To subscribe, send a message starting with '+', the first word after + will be
used as the channel to be subscribed, later words will be a first message in
case you want to do both at the same time.

    + somechannel Subscription and a first message!

To send a message without subscribing, let's use ':'.

    : somechannel Some message to all somechannel subscribers.

To unsubscribe makes sense to use -.

    - channel And a last message.
