use crate::ci::job::dag::constraint_matrix_constraint_iterator::ConstraintMatrixConstraintIterator;
use crate::ci::job::dag::{Constraint, DagError};
use crate::ci::job::Job;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct ConstraintMatrix {
    matrix: BTreeMap<(String, String), Constraint>,
    blocked_by: BTreeMap<String, BTreeSet<String>>,
    blocking: BTreeMap<String, BTreeSet<String>>,
}

impl ConstraintMatrix {
    pub fn new(jobs: &[Job], constraints: &[(String, String)]) -> Result<Self, DagError> {
        let mut matrix = BTreeMap::<(String, String), Constraint>::new();
        let mut blocks = BTreeMap::new();
        let mut blocked_by = BTreeMap::new();

        for outer in jobs {
            for inner in jobs {
                let constraint = if outer.name == inner.name {
                    Constraint::Free
                } else {
                    Constraint::Indifferent
                };
                matrix.insert((outer.name.to_string(), inner.name.to_string()), constraint);
            }

            blocks.insert(outer.name.to_string(), BTreeSet::<String>::new());
            blocked_by.insert(outer.name.to_string(), BTreeSet::<String>::new());
        }

        let job_names: Vec<String> = jobs.iter().map(|job| job.name.to_string()).collect();

        for constraint in constraints {
            if constraint.0 == constraint.1 {
                return Err(DagError::JobCannotBlockItself(constraint.1.to_string()));
            }
            if !job_names.contains(&constraint.0) {
                return Err(DagError::UnknownJobInConstraint(constraint.0.to_string()));
            }
            if !job_names.contains(&constraint.1) {
                return Err(DagError::UnknownJobInConstraint(constraint.1.to_string()));
            }
        }
        for new_constraint in constraints {
            if let Some(cons) = matrix.get_mut(new_constraint) {
                *cons = cons.constrain().map_err(|_| {
                    DagError::CycleExistsBetween(
                        new_constraint.0.to_string(),
                        new_constraint.1.to_string(),
                    )
                })?
            }
            if let Some(vec) = blocks.get_mut(&new_constraint.0) {
                vec.insert(new_constraint.1.to_string());
            }

            if let Some(vec) = blocked_by.get_mut(&new_constraint.1) {
                vec.insert(new_constraint.0.to_string());
            }
        }

        Ok(ConstraintMatrix {
            matrix,
            blocked_by: blocks,
            blocking: blocked_by,
        })
    }

    pub fn blocking(&self, link: &str) -> ConstraintMatrixConstraintIterator {
        ConstraintMatrixConstraintIterator::new(&self.blocking, link.to_string())
    }

    pub fn blocked_by(&self, link: &str) -> ConstraintMatrixConstraintIterator {
        ConstraintMatrixConstraintIterator::new(&self.blocked_by, link.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::ci::job::dag::tests::*;
    use crate::ci::job::dag::Constraint;

    #[test]
    pub fn check_direct_links() {
        let matrix = complex_pipeline().unwrap();
        assert!(matches!(
            matrix.matrix.get(&cons("build1", "build2")).unwrap(),
            Constraint::Indifferent
        ));
        assert!(matches!(
            matrix.matrix.get(&cons("build1", "test1")).unwrap(),
            Constraint::Blocked(1)
        ));
        assert!(matches!(
            matrix.matrix.get(&cons("build1", "build1")).unwrap(),
            Constraint::Free
        ))
    }
}
