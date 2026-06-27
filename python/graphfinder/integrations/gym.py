"""Gymnasium integration: a GridWorld reinforcement-learning environment backed
by a graphfinder grid, plus an **A\\* oracle**.

    from graphfinder.integrations import gym as gfgym
    env = gfgym.GridWorldEnv("S....\\n.###.\\n....G")
    obs, info = env.reset()
    action = gfgym.optimal_action(env)   # what A* would do from here

The oracle (``optimal_path`` / ``optimal_action``) makes the environment useful
for imitation learning, reward shaping, or scoring an RL agent against the
optimal policy. Requires ``gymnasium`` (``pip install "graphfinder[gym]"``).
"""
import math

from . import _require

gym = _require("gymnasium", "gym")

from graphfinder.viz import _parse_map  # noqa: E402  (light, no extra needed)

#: Action deltas. The first four are orthogonal; the last four are diagonal and
#: are only available when the env is created with ``diagonal=True``.
_MOVES = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)]


class GridWorldEnv(gym.Env):
    """A grid navigation environment over a graphfinder map.

    The agent starts at ``S`` and must reach ``G``. Walls are ``#``; a digit
    ``1``–``9`` is terrain that costs that much to enter. Observations are the
    flattened cell index (``Discrete(H*W)``); actions are moves
    (``Discrete(4)``, or ``Discrete(8)`` with ``diagonal=True``). The reward is
    the negative cost of entering the target cell (``×√2`` diagonally); an
    illegal move (into a wall or off-grid) keeps the agent in place with reward
    ``-1``. Episodes end on reaching the goal, or truncate at ``max_steps``.
    """

    metadata = {"render_modes": ["ansi"]}

    def __init__(self, map, *, diagonal=False, max_steps=None, render_mode=None):
        super().__init__()
        walls, start, goal, costs = _parse_map(map)
        if start is None or goal is None:
            raise ValueError("the map must contain a start 'S' and a goal 'G'")
        self.height = len(costs)
        self.width = len(costs[0]) if costs else 0
        # search_grid_costs treats a cell ≤ 0 as a wall.
        self.costs = [
            [0.0 if (r, c) in walls else costs[r][c] for c in range(self.width)]
            for r in range(self.height)
        ]
        self.start, self.goal = start, goal
        self.diagonal = diagonal
        self.moves = _MOVES[: (8 if diagonal else 4)]
        self.max_steps = max_steps or 4 * self.height * self.width
        self.render_mode = render_mode

        self.observation_space = gym.spaces.Discrete(self.height * self.width)
        self.action_space = gym.spaces.Discrete(len(self.moves))
        self.pos = start
        self.steps = 0

    # -- helpers --
    def _index(self, cell):
        return cell[0] * self.width + cell[1]

    def _blocked(self, cell):
        r, c = cell
        return (
            r < 0 or c < 0 or r >= self.height or c >= self.width or self.costs[r][c] <= 0.0
        )

    # -- gymnasium API --
    def reset(self, *, seed=None, options=None):
        super().reset(seed=seed)
        self.pos = self.start
        self.steps = 0
        return self._index(self.pos), {}

    def step(self, action):
        dr, dc = self.moves[int(action)]
        target = (self.pos[0] + dr, self.pos[1] + dc)
        self.steps += 1
        if self._blocked(target):
            reward = -1.0  # bumped a wall / edge: wasted step, stay put
        else:
            cost = self.costs[target[0]][target[1]]
            reward = -cost * (math.sqrt(2) if dr != 0 and dc != 0 else 1.0)
            self.pos = target
        terminated = self.pos == self.goal
        truncated = self.steps >= self.max_steps
        return self._index(self.pos), reward, terminated, truncated, {}

    def render(self):
        if self.render_mode != "ansi":
            return None
        rows = []
        for r in range(self.height):
            line = []
            for c in range(self.width):
                cell = (r, c)
                if cell == self.pos:
                    line.append("A")
                elif cell == self.goal:
                    line.append("G")
                elif self.costs[r][c] <= 0.0:
                    line.append("#")
                else:
                    line.append(".")
            rows.append("".join(line))
        return "\n".join(rows)


def optimal_path(env):
    """The A\\* shortest path of cells from the agent's current position to the
    goal (``None`` if unreachable)."""
    import graphfinder as gf

    r = gf.search_grid_costs(
        env.costs, start=env.pos, goal=env.goal, algorithm="astar", diagonal=env.diagonal
    )
    return r.path


def optimal_action(env):
    """The action index A\\* would take from the agent's current position
    (``None`` at the goal or if unreachable)."""
    path = optimal_path(env)
    if not path or len(path) < 2:
        return None
    (r0, c0), (r1, c1) = path[0], path[1]
    return env.moves.index((r1 - r0, c1 - c0))
