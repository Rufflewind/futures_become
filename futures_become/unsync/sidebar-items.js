initSidebarItems({"fn":[["becomer","Create a new future that has the ability to “run” a subfuture that supports tail-call elimination.  This introduces a `Becomer` context."],["becomes","A `Future` that simply becomes its contained value.  This only makes sense when run under a `Becomer` context."],["err","A `Future` that simply raises an error.  Replaces `futures::future::err` in `Becomer` contexts."]],"type":[["BoxBecomer","An alias for a `Becomer` that is boxed but not `Send`."],["BoxFuture","An alias for a boxed `Future` that is not `Send`."]]});