"""PyTorch integration: use a *learned* model as an A\\* heuristic.

The bridge is framework-agnostic — it just calls ``model(encode(node, goal))`` and
coerces the result to ``float`` — so it works with a PyTorch ``nn.Module``, a
NumPy function, or a scikit-learn regressor. The canonical use (and the
[tutorial](../tutorials/learned-heuristic.md)) trains a small neural net to
predict the remaining cost and plugs it into A\\*:

    from graphfinder.integrations import torch as gft
    h = gft.as_heuristic(model, encode=lambda node, goal: featurize(node, goal))
    gf.search(maze, algorithm="astar", heuristic=h)

A learned heuristic is usually **not admissible**, so A\\* may return a slightly
sub-optimal path — but it often expands far fewer nodes. Use ``weighted_astar``
or verify cost if you need a bound.
"""


def as_heuristic(model, encode, no_grad=True):
    """Turn a learned ``model`` into a ``h(node, goal) -> float`` callable.

    Args:
        model: any callable mapping encoded features to a scalar-like value
            (e.g. a PyTorch ``nn.Module``). Its output is coerced with ``float``.
        encode: ``encode(node, goal)`` returning the model's input (a tensor,
            array, …). For grids, ``node``/``goal`` are ``(row, col)`` tuples; for
            graphs they are node ids.
        no_grad: if PyTorch is loaded, evaluate inside ``torch.no_grad()`` for
            speed. Ignored for non-torch models.

    Returns:
        A callable suitable for the ``heuristic=`` argument of any graphfinder
        search.
    """
    torch_mod = None
    if no_grad:
        import sys

        torch_mod = sys.modules.get("torch")

    if torch_mod is not None:
        def h(node, goal):
            with torch_mod.no_grad():
                return float(model(encode(node, goal)))
    else:
        def h(node, goal):
            return float(model(encode(node, goal)))

    return h
