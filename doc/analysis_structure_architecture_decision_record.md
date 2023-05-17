## Introduction

* Prologue (Summary)
* Discussion (Context)
* Options
* Solution (Decision)
* Consequences (Results)

## Specifics ##

* Prologue (Summary)
    * Statement to summarize:
        * In the context of the choice of files analysis structure<br>
          facing rust strict rules regarding memory management,<br>
          we decided to store all Analysis into a Hashmap with an unique id,<br>
          in order to have direct parents/children access between them,<br> 
          accepting the loss of real file system organization.<br>


* Discussion (Context)
    * Produced folders and files analysis must have access to their parents and children <br>
        in order to be able to (among other things) update parents analysis after the insertion of a file analysis.
    * This implies sharing data between several analysis <br>
      How to organize analysis structure and relationships between them <br>
      respecting the strict Rust rules concerning ownership and lifetime?


* Options
    * Directional analysis structures (Analysis system reproduce files system)
  
        * Reference counter within Analysis fields, with Rc<RefCell<Analysis>>> for both parent and children 
          * Pros: Allow precise management of ownership between parents/children in analysis tree, <br>
                      keeping intact the real data organization, with internal mutability 
          * Cons: Very technical, complex, unintelligible
          
        * Precise management of lifetimes within Analysis fields,<br>
      with parent: Option<& 'a RefCell<Analysis<'a>>>,<br>
         and children: Option<BTreeMap<FileName, RefCell<Analysis<'a>>>>
            * Pros: sames as above
            * Cons: same as above, uncertainties about lifetime of children analysis 
          
        * Parent owns children, direct parent-to-children access, and XPath-like system to access parent
            * Pros: Easy to navigate in the tree and find any element
            * Cons: Every access to analysis must start from the root, with no direct access to parent
  
    * Indirectionnal analysis structures (flat organization, loss of real file system)
  
        * Analysis tree into a BTreeMap < NodeId, Node { analysis, parent, children } >
            * Pros: No ownership/lifetime issues, direct access to parent/children
            * Cons: extra features compare to HashMap seem unnecessary (ordered keys)
          
        * Analysis tree into a HashMap < NodeId, Node { analysis, parent, children } >
            * Pros compared to BTreeMap: Operation are usually faster, use less memory 
            * Cons compared to BtreeMap: n/a
          
        * Analysis tree into a Vec < Nodes { analysis, parent, children } > with vector index
            * Pros: Very simple, no ownership/lifetime issues, direct access to parent/children
            * Cons: Highly error-prone
      
        * All Analysis owned by a single Arena (Arena Memory Allocation)
          * Pros: No ownership/lifetime issues
          * Cons: less memory efficiency if there are frequent analysis updates, we can do the same with maps  


* Solution

    * Which option has been chosen
        * The best option would be reproducing the tree-like file system organization for analysis, <br>
        and managing perfectly and without pain Rust specific concepts about memory management.<br>
        As we are currently unable to do so, an indirectionnal approach seems unavoidable. 
        The use of HashMap seems to be a simple, robust and efficient strategy for our purpose. 


* Consequences
    Not known for now.