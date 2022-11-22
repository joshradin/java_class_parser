//! Provides mechanisms to inspect the inheritance structure of a class

use crate::error::{Error, ErrorKind};
use crate::structures::FQName;
use crate::{FQNameBuf, JavaClass, JavaClassParser};
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// A graph representing interfaces and super classes of a given root class.
#[derive(Debug)]
pub struct InheritanceGraph {
    graph: DiGraph<FQNameBuf, InheritKind>,
    mapping: HashMap<FQNameBuf, (JavaClass, NodeIndex)>,
    root: FQNameBuf,
}

/// How a given type inherits another type
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum InheritKind {
    /// Used from class to class dependencies
    Extends,
    /// An interface that this type implements
    Implements,
}

impl InheritanceGraph {
    fn new(class: JavaClass) -> Self {
        let fcq = class.this().to_owned();
        let mut graph = DiGraph::new();
        let index = graph.add_node(fcq.clone());
        let map = HashMap::from([(fcq.clone(), (class, index))]);
        Self {
            graph,
            mapping: map,
            root: fcq,
        }
    }

    /// Adds a class. Returns true only if this class hasn't been added yet.
    fn add_class(&mut self, class: JavaClass) -> bool {
        if self.mapping.contains_key(class.this()) {
            return false;
        }

        let index = self.graph.add_node(class.this().to_owned());
        self.mapping.insert(class.this().to_owned(), (class, index));
        true
    }

    /// add inheritance. returns true only if both classes are in the graph and an existing inheritance
    /// doesn't already exist
    fn add_inheritance(&mut self, class: &FQName, inherits: &FQName, ty: InheritKind) -> bool {
        let Some(&(_, class)) = self.mapping.get(class) else {
            eprintln!("doesn't contain class {}", class);
            return false;
        };
        let Some(&(_, inherits)) = self.mapping.get(inherits) else {
            eprintln!("doesn't contain class {}", inherits);
            return false;
        };

        if self.graph.contains_edge(class, inherits) {
            eprintln!(
                "already contains edge between {} -> {} ({:?})",
                self.graph[class],
                self.graph[inherits],
                self.graph.find_edge(class, inherits).unwrap()
            );
            return false;
        }

        self.graph.add_edge(class, inherits, ty);
        true
    }

    fn get_class(&self, node_index: NodeIndex) -> &JavaClass {
        let name = &*self.graph[node_index];
        let (class, _) = self
            .mapping
            .get(name)
            .expect("index didn't correspond to known class");
        class
    }

    /// Gets the classes that this class extends or interfaces it implements that are present on
    /// the originating classpath. Order is determined in breadth first order.
    pub fn inherits<F: AsRef<FQName>>(
        &self,
        fqn: F,
    ) -> Result<Vec<(&JavaClass, InheritKind)>, Error> {
        let fq_name = fqn.as_ref();
        if !self.mapping.contains_key(fq_name) {
            return Err(Error::from(ErrorKind::NoClassFound(
                fq_name.to_fqname_buf(),
            )));
        }

        let mut outout = vec![];
        let mut visited: HashSet<&FQName> = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(fq_name);
        while let Some(ptr) = queue.pop_front() {
            if !visited.contains(ptr) {
                let (_, from_index) = self.mapping[ptr];
                let inherits = self.graph.edges(from_index);
                for edge in inherits {
                    let &inherit = edge.weight();
                    let to_class = self.get_class(edge.target());
                    if !visited.contains(to_class.this()) {
                        outout.push((to_class, inherit));
                        queue.push_back(to_class.this())
                    }
                }

                visited.insert(ptr);
            }
        }

        Ok(outout)
    }
}

/// Inspects a class to create an inheritance graph
pub fn inspect(class: &JavaClass, parser: &JavaClassParser) -> Result<InheritanceGraph, Error> {
    let mut graph = InheritanceGraph::new(class.clone());

    let mut stack = vec![];
    stack.push(class.clone());
    while let Some(class) = stack.pop() {
        let super_class = match parser.find_super(&class) {
            Ok(o) => {
                Some(o)
            }
            Err(e) => {
                if let ErrorKind::NoClassFound(_) = e.kind() {
                    None
                } else {
                    return Err(e)
                }
            }
        };
        if let Some(super_class) = super_class {
            let super_class_name = super_class.this().to_fqname_buf();
            if graph.add_class(super_class.clone()) {
                stack.push(super_class);
            }
            if !graph.add_inheritance(class.this(), &super_class_name, InheritKind::Extends) {
                return Err(Error::new(ErrorKind::AddingInheritanceFailed(
                    class.this().to_fqname_buf(),
                )));
            }
        }
        let interfaces = parser.find_interfaces(&class)?;
        for interface in interfaces {
            let interface_name = interface.this().to_fqname_buf();
            if graph.add_class(interface.clone()) {
                stack.push(interface);
            }
            if !graph.add_inheritance(class.this(), &interface_name, InheritKind::Implements) {
                return Err(Error::new(ErrorKind::AddingInheritanceFailed(
                    class.this().to_fqname_buf(),
                )));
            }
        }
    }

    Ok(graph)
}
