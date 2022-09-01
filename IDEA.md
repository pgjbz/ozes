## IDEA

The idea of this queue service is being the simplest possibly.

To create a consumer just send to the server:

SUBSCRIBE < queue_name > WITH GROUP < group name >;

To create a publisher just send to the server:

PUBLISHER < queue_name >;

And with the PUBLISHER just send:

MESSAGE "message here";
