# xcrypto

[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE-MIT)

xcrypto is a trading system with a Client/Server architecture. It is implemented based on rust tokio and can support up to 65536 strategies running simultaneously. It communicates with strategies using WebSocket, automates trading by forwarding exchange quotes, accepting order requests from strategies, and distributing order returns from exchanges. This project mainly consists of two subprojects: the **trading system** and the **strategy package**. The trading system implements Binance's spot trading and usdt future trading. The strategy package is implemented using Rust and Python, strategies can written entirely in python.

## Implement strategy

```python
from pyalgo import *


class Demo:
    """"""

    def __init__(self, sub: DepthSubscription):
        self.sub = sub
        # use to send/kill order
        self.smtord = SmartOrder(sub)

        self.fin = False

        # add trading phase if you need
        # otherwise self.sub.phase is always Phase.UNDEF
        self.sub.add_phase(0, 0, 0, Phase.OPEN)
        self.sub.add_phase(16, 0, 0, Phase.CLOSE)

        # set callback
        self.sub.on_data = self.on_depth
        self.sub.on_order = self.on_order

    def on_order(self, order: Order):
        info(f"{order}")

    def on_depth(self, depth: Depth):
        info(f"{self.sub.datetime}, {self.sub.symbol}")

        if self.fin:
            self.smtord.kill()

        match self.sub.phase:
            case Phase.OPEN:
                if self.sub.net == 0:
                    self.smtord.send(
                        self.sub.bid_prc(5),
                        10,
                        Side.BUY,
                        OrderType.LIMIT,
                        Tif.GTC,
                    )
                    self.fin = True

            case Phase.CLOSE:
                # todo
                pass


if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    sub = session.subscribe("dogeusdt", "depth")
    demo = Demo(sub)
    eng.run()
```

## Risk Warning

It is a personal project, use at your own risk. I will not be responsible for your investment losses (and gains).
Cryptocurrency investment is subject to high market risk.

## Prerequisites

- rustc >= 1.80 
- python >= 3.10
- maturin >= 1.5.1
- sqlite3 >= 3.41.2

### Install rust

``` shell
url --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install maturin

```shell
pip install maturin
```

## Introduction

This project consists of two subprojects. One is a Rust binary project, which serves as a bridge between trading strategies and the exchange. The other is a hybrid project of Rust and Python, which aims to provide convenience for implementing strategies.

![xcrypto](static/xcrypto.svg)

An important concept is **Session**. Each Session represents an instance of a strategy. At any given time, a session can only be used by one strategy. The distribution of orders and the recording of positions are differentiated based on the session.

Here is an example

```python
eng = Engine()
session = eng.make_session(
    addr="ws://localhost:8111", session_id=1, name="test", trading=True
)
sub = session.subscribe("btcusdt","depth")
```

In this example, session 1 is being used, and the `btcusdt` is subscribed through this session, **its position holdings also being saved to session 1**.

You can also set `trading=False`, in which case, the session will not support trading. Your strategy will not be able to send orders or cancel orders, but it can still receive market data.

## Build rust binary

For simplicity, the system only supports trading for spot account and futures cross-margin account, and does not support trading for isolated-margin accounts.

The supported market data includes depth and kline data for all periods, which is sufficient for most strategies to use.

### Spot

```shell
➜  ~ cd xcrypto
➜  ~ ls
➜  ~ Cargo.lock  Cargo.toml  binance  logger  private_key.pem  pyalgo  spot.json  src  usdt.json
➜  ~ cd binance/spot
```
build debug

```shell
➜  ~ cargo build
```

or release

```shell
➜  ~ cargo build -r
```

You will find the binary file in target/debug or targer/release

### USDT future

```shell
➜  ~ cd xcrypto
➜  ~ ls
➜  ~ Cargo.lock  Cargo.toml  binance  logger  private_key.pem  pyalgo  spot.json  src  usdt.json
➜  ~ cd binance/usdt
```

build debug

```shell
➜  ~ cargo build
```

or release

```shell
➜  ~ cargo build -r
```

You will find the binary file in target/debug or targer/release


## Run

Before you run the program, two files is required, one is the configuration file, and the other is your private key.

For these two files, you can refer to `usdt.json` and `spot.json` for the configuration file, and `private_key.pem` for the private key.

After compilation is completed, jump to the target/debug or target/release. **Before you run the program, you need to do the following things**.
- **Set position mode to Single-Side Position**
- **Move configuration file, private key and binary file to the same directory**

You can run spot trading by this command

```shell
./spot -c=spot.json -l=info
```

The argument `-c=spot.json` is the abbreviation of `--config=spot.json`, representing the path to the configuration file. The `-l=info` is the abbreviation of `--level=info`, indicating the log level, this parameter is optional, its default value is `info`. The configuration file looks like this.

```json
{
    "apikey":"your api key",
    "pem": "private_key.pem",
    "margin": false,
    "local": "ws://localhost:8111"
}
```

- **apikey** can be generated through the Binance.
- **pem** is the path where your private key is located, which in this example is `private_key.pem` located in the current directory.
- **margin** determine to use spot account or cross-margin account
- **local** is the websocket address bind to, and the strategy will communicate with the trading system by connecting to this address


For usdt future, it is similar to spot trading.

```shell
./usdt -c=usdt.json -l=info
```

The configuration file looks like this.

```json
{
    "apikey": "your api key",
    "pem": "private_key.pem",
    "local": "ws://localhost:8111"
}
```

The meaning of each field is the same as the `spot.json`

## Position

After you run the binary, pos.db will appear in the current directory where you executed it. This file is a SQLite3 database that is used to store the position holdings for different sessions.

If you need to modify the position recorded by the system, you can make changes to pos.db using SQL. After completing the modifications, simply restart the system.

## Python strategy package


```shell
➜  ~ cd xcrypto
➜  ~ ls
➜  ~ Cargo.lock  Cargo.toml  binance  logger  private_key.pem  pyalgo  spot.json  src  usdt.json
➜  ~ cd pyalgo
```

install debug

```shell
➜  ~ maturin develop
```

install release

```shell
➜  ~ maturin develop -r
```

You can also package as wheels

```shell
➜  ~ maturin build 
```

or 

```shell
➜  ~ maturin build -r
```
The `.whl` file is located in the target/wheels


## Implement strategy

In the `pyalgo/example` directory, there are some demos. To implement any strategy, the following classes is all you need.

- **Engine**, used to create sessions. DepthSubscription and BarSubscription are subscribed through the created session. Here is an example. 

```python
from pyalgo import *

if __name__ == "__main__":
    # The only argument is interval
    # Because the underlying WebSocket is non-blocking 
    # In order to avoid wasted CPU cycles, an appropriate sleep interval is required. 
    # You can also set it to 0, and it will become a busy loop.
    eng = Engine(0.0001)
    # create session
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )   
    # subscribe depth, you can also subscribe kline by session.subscribe("dogeusdt","kline:1m)
    sub = session.subscribe("dogeusdt", "depth")
    # set callback
    # when you recv depth, it will print data
    sub.on_data = lambda x:print(x)
    eng.start()
```

- **DepthSubscription**, with this class, you can obtain the depth of market data, contract information, and net position.

- **BarSubscription**, it is similar to DepthSubscription, and can be obtained by calling session.subscribe(symbol, "kline:1m"). all kline on Binance are supported

- **SmartOrder**, this class is used for sending orders and cancelling orders. It can only manage one order at a time. **When an order is in an is_active state (pending or partially traded)**, it cannot send a new order. Here is an example. 

```python
from pyalgo import *


class Demo:
    """"""

    def __init__(self, sub: DepthSubscription):
        self.sub = sub
        self.smtord = SmartOrder(sub)

        self.fin = False

        self.sub.on_data = self.on_depth

    def on_order(self, order: Order):
        info(f"{order}")

    def on_depth(self, _: Depth):
        if self.fin:
            self.smtord.kill()
            return
        
        if self.smtord.is_active:
            # if our prc < best bid, kill it
            if self.smtord.price < self.sub.bid_prc(0):
                self.smtord.kill()

        else:
            self.smtord.send(
                self.sub.bid_prc(0), 10, Side.BUY, OrderType.LIMIT, Tif.GTC
            )
            self.fin = True


if __name__ == "__main__":
    eng = Engine(0.001)
    session = eng.make_session(
        addr="ws://localhost:8111", session_id=1, name="test", trading=True
    )

    sub = session.subscribe("dogeusdt", "depth")
    demo = Demo(sub)
    eng.run()
```

## Donation

Open source is not easy. Please give the author some encouragement. Treat the author a cup of Mixue or Luckin Coffee, your Issues will be resolved first.

![alipay](static/alipay.jpeg) 
![alipay](static/wechat.jpeg)
