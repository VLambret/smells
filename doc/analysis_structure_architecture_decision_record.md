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
      respecting the strict Rust rules concerning ownership and lifetime   
* Options
    * Directional data structures
        * Rc<RefCell<Node>>> 
            * Pros : Allow precise management of ownership between parents/children in analysis tree, <br>
                keeping intact the real data organization
            * Cons : the difficulty and technicality of this approach suggests that it is not the ideal solution 
        * Arena Memory Allocation
            * Pros : No ownership/lifetime issues
    * Indirectional data structures
        * BTreeMap<NodeId, Node { analysis, parent, children }>
            * Pros : No ownership/lifetime issues, direct access to parent/children
            * Cons : Loss of real data organization
        * Vec<Nodes { analysis, parent, children }> with vector index
            * Pros : Very simple, No ownership/lifetime issues, direct access to parent/children
            * Cons : Highly error-prone, Loss of real data organization 
        * XPath-like system
            * Cons : Every access to analysis must start from root, no access to parent,<div>
                Loss of real data organization

* Solution
    * Which option has been chosen
    * Explains how the decision will solve the problem.
* Consequences
    * Explains the results of the decision over the long term.
    * Did it work, not work, was changed, upgraded, etc. or more difficult to do because of this change?