"""The stamina and food model of survival-mode design 12, stepped at 10 ms.

A planning instrument: it prints the design's worked examples (a minute of
sprinting, a 200 m swim, a world day of normal play) from the proposed
`assets/config/survival.ron` numbers, so the table in the design and the
curve on docs/mockups/survival.html#stamina come from one model. It is not
game code; pbd_core::survival is written from the design when it is built.

    python3 tools/survival/stamina_model.py
"""

P = dict(
    regen=12.0,  # pts/s
    delay=1.0,  # s with nothing spending before the refill starts
    c_low=0.02,  # fp a point at or below half
    c_high=0.05,  # fp a point at full, linear from half
    strain=1.5,  # refill cost multiplier while strain is owed
    sprint=(1.0, 0.03),  # pts/s, fp/s
    swim=(0.8, 0.025),
    jump=2.0,  # pts
    climb=1.5,  # pts a metre
    base=100 / (75 * 60),  # fp/s standing about
)


def cost(s):
    if s <= 50:
        return P["c_low"]
    return P["c_low"] + (P["c_high"] - P["c_low"]) * (s - 50) / 50


class Body:
    def __init__(self, food=80.0):
        self.s, self.strain, self.food0, self.spent, self.effort, self.since = 100.0, 0.0, food, 0.0, 0.0, 99.0

    def step(self, dt, mode=None):
        self.spent += P["base"] * dt
        if mode in ("sprint", "swim"):
            pts, fp = P[mode]
            self.s = max(0.0, self.s - pts * dt)
            self.strain = min(100 - self.s, self.strain + pts * dt)
            self.spent += fp * dt
            self.effort += fp * dt
            self.since = 0.0
            return
        self.since += dt
        if self.since < P["delay"] or self.s >= 100:
            return
        food = self.food0 - self.spent
        rate = P["regen"] * (1.0 if food >= 50 else 0.4 + 0.6 * max(food, 0) / 50)
        r = min(rate * dt, 100 - self.s)
        k = P["strain"] if self.strain > 0 else 1.0
        c = cost(self.s + r / 2) * r * k
        self.spent += c
        self.effort += c
        self.s += r
        self.strain = max(0.0, self.strain - r)

    def spend(self, pts):
        self.s = max(0.0, self.s - pts)
        self.since = 0.0

    def run(self, secs, mode=None, dt=0.01):
        for _ in range(int(round(secs / dt))):
            self.step(dt, mode)


def day(b):
    t = 0
    for _ in range(18):
        b.run(20, "sprint"); b.run(20); t += 40
    for _ in range(2):
        b.run(60, "swim"); b.run(10); t += 70
    for _ in range(40):
        b.spend(P["jump"]); b.run(5); t += 5
    for _ in range(30):
        b.spend(P["climb"]); b.run(3); t += 3
    b.run(2880 - t)


CASES = [
    ("a minute of sprinting, then 10 s", lambda b: (b.run(60, "sprint"), b.run(10))),
    ("70 s walking", lambda b: b.run(70)),
    ("a 200 m swim (50 s), then 6 s", lambda b: (b.run(50, "swim"), b.run(6))),
    ("a world day of normal play", day),
    ("a world day standing about", lambda b: b.run(2880)),
]

if __name__ == "__main__":
    for name, f in CASES:
        b = Body()
        f(b)
        print(f"{name}: {b.spent:.2f} fp spent, {b.effort:.2f} fp of it the effort, stamina {b.s:.0f}")
