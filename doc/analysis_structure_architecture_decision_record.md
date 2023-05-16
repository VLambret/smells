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
        * In the context of smells project analysis structure<br>
          facing rust problems regarding the choice of architecture,<br>
          we decided to choose a solution using Hashmaps to build and update
          the tree, accepting the loss of the real data organization.<br>


* Discussion (Context)
    * Produced analysis must have direct or indirect access to their parent and children <br>
        to reproduce the file system organization, <br>
        and be able to update them after the insertion of a file analysis.
    * How to organize analysis structure and relationships between them <br>
        respecting the strict Rust rules concerning ownership and lifetime?


* Options
    * Directional data structures 
  
        * Analysis tree -> Rc<RefCell<Node>>> for both parent and children
            * Pros: Allow precise management of ownership between parents/children in analysis tree, <br>
                keeping intact the real data organization, with internal mutability
            * Cons: The difficulty and technicality of the solution
          
        * Analysis tree -> parent: Option<& 'a RefCell<Analysis<'a>>>,
                           content: Option<BTreeMap<String, RefCell<Analysis<'a>>>>,
            * Pros: access to parents from children
            * Cons: uses of Refcell and lifetimes (we are not sure about what's the lifetime of analysis
              in content) that could add complexity
  
    * Directionless data structures (loss of real file structure)
  
        * Analysis tree -> BTreeMap < NodeId, Node { analysis, parent, children } >
            * Pros: No ownership/lifetime issues, direct access to parent/children
            * Cons: extra features (keeping the order of entries) are not really necessary
          
        * Analysis tree -> HashMap < NodeId, Node { analysis, parent, children } >
            * Pros compared to BTreeMap: Operation are usually faster, use less memory 
            * Cons compared to BtreeMap: n/a
          
        * Analysis tree -> Vec < Nodes { analysis, parent, children } > with vector index
            * Pros: Very simple, No ownership/lifetime issues, direct access to parent/children
            * Cons: Highly error-prone
          
        * XPath-like system
            * Pros: Easy to navigate in the tree and find any element
            * Cons: Every access to analysis must start from the root, with no direct access to parent
          
        * Analysis tree -> Arena Memory Allocation, all nodes/analysis owned by a single Arena
          * Pros: No ownership/lifetime issues
          * Cons: less memory efficiency if there are frequent updates in nodes


* Solution

    * Which option has been chosen
        * The best option would be to keep the tree structure by limiting the uses of rust concepts like lifetimes, 
        reference-counting pointer, refcell... We can't do that for now. To have less complexity in the code,
        we must choose a directionless solution. We need something efficient(we need to access nodes quickly), 
        robust and the order of entries is not useful. The more suitable option seems to be the HashMap.

* Consequences
    Not known for now.