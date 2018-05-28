/// An AST node that can be visited
pub trait Visit {
    fn visit<'ast, V>(&'ast self, &mut V);

    fn visit_mut<V>(&mut self, &mut V);
}

impl<T> Visit for Box<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) {
        (**self).visit(v)
    }

    fn visit_mut<V>(&mut self, v: &mut V) {
        (**self).visit_mut(v)
    }
}

impl<T> Visit for Option<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) {
        for i in self {
            i.visit(v)
        }
    }

    fn visit_mut<V>(&mut self, v: &mut V) {
        for i in self {
            i.visit_mut(v)
        }
    }
}

impl<T> Visit for Vec<T>
where
    T: Visit,
{
    fn visit<'ast, V>(&'ast self, v: &mut V) {
        for i in self {
            i.visit(v)
        }
    }

    fn visit_mut<V>(&mut self, v: &mut V) {
        for i in self {
            i.visit_mut(v)
        }
    }
}

/// Directs the visitor to continue processing the children of the
/// current code or not.
#[derive(Debug, PartialEq)]
pub enum Control {
    Continue,
    Break,
}
