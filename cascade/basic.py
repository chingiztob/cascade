from cascade import _cascade_core as core

def foo():
    return "cock"

def return_string_wrapper():
    """wtf"""
    return core.return_string()

def demo_wrapper(graph):
    return core.demo(graph)

def create_graph():
    return core.create_graph()

