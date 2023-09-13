#! /usr/bin/python3

from typing import List

import graphviz


class Item:
    def __init__(self, id: str, label: str, dependencies: str = '', size: int = None, done: bool = False):
        self.id = id
        self.label = label
        self.dependencies = list(filter(None, dependencies.split(' ')))
        self.done = done
        self.size = size

class Direction:
    def __init__(self, name: str, items: List[Item]):
        self.name = name
        self.items = items


def print_graphiz(backlog):
    g = graphviz.Digraph(comment='Backlog', node_attr={'color': 'lightblue2', 'style': 'filled'})
    explicit_edges = []

    for direction in backlog:
        cluster_name = f'cluster_{direction.name}'.replace(' ', '')
        with g.subgraph(name=cluster_name) as d:
            d.attr(label=direction.name)
            previous = None
            item: Item
            for item in direction.items:
                color = 'lightblue2'
                if item.done:
                    color = 'lightgreen'
                d.node(item.id, item.label, color=color)

                # Create default links
                if previous:
                    d.edge(previous.id, item.id)
                previous = item

                for dependency in item.dependencies:
                    explicit_edges.append([dependency, item.id])

    for link in explicit_edges:
        g.edge(link[0], link[1])

    print(g.source)


backlog = [

    ############################################################
    # DEV ENVIRONMENT
    ############################################################

    Direction('Rust basics',
              items=[Item(id='RB1', label='Helloworld', done=True),
                     Item(id='RB2', label='fizzbuzz', dependencies='UT1', done=True),
                     Item(id='RB3', label='ls', done=True),
                     Item(id='RB4', label='print branch name using git lib', done=True),
                     Item(id='RB5', label='print HEAD meta-data (author, title, date', done=True),
                     Item(id='RB6', label='git log --oneline', done=True),
                     ]),
    Direction('Unit Tests',
              items=[
                  Item(id='UT1', label='Unit Tests', done=True),
                  Item(id='UT2', label='Parameterized tests', done=True),
                  Item(id='UT3', label='test main', done=True),
                  Item(id='UT4', label='Setup/TearDown', done=True),
                  Item(id='UT5', label='Mock internal functions', done=True),
                  Item(id='UT6', label='Mock system calls'),
                  ]),
    Direction('E2E Tests',
              items=[
                  Item(id='E2ET1', label='End to End tests', done=True),
                  Item(id='E2ET2', label='BDD tests', done=True),
                  ]),
    Direction('Tools',
              items=[
                  Item(id='TOOL1', label='Linter/Formater'),
                  Item(id='TOOL2', label='Static Analysis'),
                  Item(id='TOOL3', label='Code coverage', dependencies='UT1'),
                  Item(id='TOOL4', label='Performances', done=True),
                  ]),
    Direction('Packaging',
              items=[
                  Item(id='PACKAGE0', label='Semantic Release'),
                  Item(id='PACKAGE1', label='Linux', done=True),
                  Item(id='PACKAGE2', label='Windows', done=True),
                  Item(id='PACKAGE3', label='Mac', done=True),
                  ]),
    Direction('CI',
              items=[
                  Item(id='CI1', label='Compilation', done=True),
                  Item(id='CI2', label='Tests', dependencies='UT1', done=True),
                  Item(id='CI3', label='E2E Tests', dependencies='E2ET1', done=True),
                  Item(id='CI4', label='Linter', dependencies='TOOL1'),
                  Item(id='CI5', label='Static Analysis', dependencies='TOOL2'),
                  Item(id='CI6', label='Performances', dependencies='TOOL3'),
                  Item(id='CI7', label='Notifications', done=True),
                  ]),

    ############################################################
    # METRICS
    ############################################################

    Direction('Size Analysis',
              items=[
                  Item(id='SIZE1', label='ls git files only', dependencies='RB3'),
                  Item(id='SIZE2', label='Compute file size', done=True),
                  Item(id='SIZE3', label='Structured JSON format', done=True),
                  Item(id='SIZE4', label='Directory score', done=True),
                  ]),
    Direction('Social complexity',
              items=[
                  Item(id='SOCIAL1', label='Compute number of authors', dependencies='SIZE4 RB4', done=True),
                  ]),
    Direction('Activity',
              items=[
                  Item(id='ACTIVITY1', label='# of changes last year', dependencies='RB4'),
                  Item(id='ACTIVITY2', label='Follow renames', dependencies='RB4'),
                  Item(id='ACTIVITY3', label='Ignore some commits ?', dependencies='RB4'),
                  ]),
    Direction('Count bugs',
              items=[
                  Item(id='BUG1', label='Count fix bug commits', dependencies='ACTIVITY1'),
                  ]),


    ############################################################
    # UI
    ############################################################

    Direction('CLI',
              items=[
                  Item(id='CLI1', label='UX design'),
                  Item(id='CLI2', label='--usage', done=True),
                  Item(id='CLI3', label='--version', done=True),
                  Item(id='CLI4', label='Size', dependencies='SIZE4', done=True),
                  Item(id='CLI5', label='Social complexity', dependencies='SOCIAL1', done=True),
                  Item(id='CLI6', label='Activity', dependencies='ACTIVITY1'),
                  Item(id='CLI7', label='Bugs', dependencies='BUG1'),
                  ]),
    Direction('Prototype Native GUI',
              items=[
                  Item(id='GUI1', label='Application with menu', dependencies='SIZE3'),
                  Item(id='GUI2', label='Draw a rectangle and a circle'),
                  Item(id='GUI3', label='Fold and unfold rectangle animation'),
                  Item(id='GUI4', label='Overlay on mouse over'),
                  ]),
    Direction('Prototype Web GUI',
              items=[
                  Item(id='WEBGUI1', label='Application with menu', dependencies='SIZE3'),
                  Item(id='WEBGUI2', label='Draw a rectangle and a circle'),
                  Item(id='WEBGUI3', label='Fold and unfold rectangle animation'),
                  Item(id='WEBGUI4', label='Overlay on mouse over'),
                  ]),
    Direction('UI',
              items=[
                  Item(id='UI0', label='Choose UI solution', dependencies='GUI4 WEBGUI4'),
                  Item(id='UI1', label='UX design'),
                  Item(id='UI2', label='Size', dependencies='SIZE4'),
                  Item(id='UI3', label='Social complexity', dependencies='SOCIAL1'),
                  Item(id='UI4', label='Activity', dependencies='ACTIVITY1'),
                  Item(id='UI5', label='Bugs', dependencies='BUG1'),
                  ]),
    Direction('Optimization',
              items=[
                  Item(id='OPTI1', label='Linux-kernel timings', dependencies='TOOL4', done=True),
                  Item(id='OPTI2', label='optimize bottlenecks', done=True),
                  ]),

]

print_graphiz(backlog)
