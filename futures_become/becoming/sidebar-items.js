initSidebarItems({"fn":[["becomer","Create a new future that has the ability to “run” a subfuture that supports tail-call elimination.  This introduces a `Becomer` context."],["becomes","A `Future` that simply becomes its contained value.  This only makes sense when run under a `Becomer` context."],["err","A `Future` that simply raises an error.  Replaces `futures::future::err` in `Becomer` contexts."]],"mod":[["unsync","Analog of the outer module, but for non-`Send` futures."]],"struct":[["Becomer","Dispatcher that can perform dynamic tail-call elimination."]],"trait":[["Becoming","Controls the type of trait object used to dispatch tail calls."]],"type":[["BoxBecomer","An alias for a `Becomer` that is boxed and `Send`."],["BoxFuture","An alias for a boxed `Future` that is `Send`."]]});