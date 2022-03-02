# SRON

Sron is an experimental, bare-bones latency testing tool.


## Running

`sron` cycles through its positional argument list for each `GET` call.

One might write:

```sh
sron -d10ms -p1ms -- "https://google.com"
```

which will send 10 requests to google (10ms duration with a 1ms period.)

Or

```sh
sron -d10ms -p1ms -- "https://google.com" "https://yahoo.com"
```

which will send 5 requests to google, and 5 requests to yahoo, interleaved.

A large number of requests can be cycled through by reading from file such as:

```sh
sron -d10ms -p1ms -- $(cat my_calls.txt)
```

where the contexts of `my_call.txt` may be:

```text
https://google.com
https://yahoo.com
https://bing.com
```
