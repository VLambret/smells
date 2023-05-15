# Decision record template for Alexandrian pattern

## Introduction

* Prologue (Summary)
* Discussion (Context)
* Options
* Solution (Decision)
* Consequences (Results)

## Specifics ##

* Prologue (Summary)
    * Statement to summarize:
        * In the context of (use case)<br>
          facing (concern)<br>
          we decided for (option)<br>
          to achieve (quality)<br>
          accepting (downside).
* Discussion (Context)
    * Explains the forces at play (technical, political, social, project).
    * How to organize analysis structure and relationships between them <br>
      respecting the strict Rust rules concerning ownership   
* Options
    * Directional data structures
        * Rc<RefCell<Node>>> 
            * Pros : Allow management of ownership between parents/children in analysis tree
        * Arena Memory Allocation
            * Pros : No ownership / lifetime issues
    * Indirectional data structures
        * BTreeMap<NodeId, Node { analysis, parent, children }>
            * Pros : No ownership / lifetime issues, direct access to parent/children
        * Vec<Nodes { analysis, parent, children }> with vector index
            * Pros : Very simple, No ownership/lifetime issues, direct access to parent/children
            * Cons : Highly error-prone 
        * XPath-like system
            * Cons : Every access to analysis must start from root, no access to parent

* Solution
    * Which option has been chosen
    * Explains how the decision will solve the problem.
* Consequences
    * Explains the results of the decision over the long term.
    * Did it work, not work, was changed, upgraded, etc. or more difficult to do because of this change?