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
    * Produced analysis must have direct or indirect access to their parent and children <br>
        in order to reproduce the file system organization, <br>
        and be able to update them after insertion of a file analysis.
    * How to organize analysis structure and relationships between them <br>
        respecting the strict Rust rules concerning ownership and lifetime ?


* Options
    * Directional data structures 
        * Analysis tree -> Rc<RefCell<Node>>> for both parent and children
            * Pros : Allow precise management of ownership between parents/children in analysis tree, <br>
                keeping intact the real data organization, with internal mutability
            * Cons : The difficulty and technicality of this approach suggests that it is not the ideal solution 
          * Analysis tree -> Arena Memory Allocation, all nodes/analysis owned by a single Arena 
              * Pros : No ownership/lifetime issues
              * Cons :
  
    * Indirectional data structures (loss of real file structure)
      * Analysis tree -> BTreeMap < NodeId, Node { analysis, parent, children } >
          * Pros : No ownership/lifetime issues, direct access to parent/children
          * Cons : Are extra features compared to HashMap really necessary ?
      * Analysis tree -> HashMap < NodeId, Node { analysis, parent, children } >
          * Pros compared to BTreeMap : Operation are usually faster, use less memory 
          * Cons compared to BtreeMap : 
      * Analysis tree -> Vec < Nodes { analysis, parent, children } > with vector index
          * Pros : Very simple, No ownership/lifetime issues, direct access to parent/children
          * Cons : Highly error-prone
      * XPath-like system
          * Cons : Every access to analysis must start from root, no direct access to parent


* Solution
    * Which option has been chosen
    * Explains how the decision will solve the problem.


* Consequences
    * Explains the results of the decision over the long term.
    * Did it work, not work, was changed, upgraded, etc. or more difficult to do because of this change?