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
        print(order)

    def on_depth(self, depth: Depth):
        print(self.sub.datetime, self.sub.symbol)

        match self.sub.phase:
            case Phase.OPEN:
                if self.sub.net == 0:
                    if not self.fin:
                        if self.smtord.is_active:
                            self.smtord.kill()

                        else:
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
