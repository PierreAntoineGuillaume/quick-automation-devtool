use crate::ci::job::dag::constraint_matrix_constraint_iterator::ConstraintMatrixConstraintIterator;
use crate::ci::job::dag::{Constraint, DagError};
use crate::ci::job::Job;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct ConstraintMatrix {
    blocked_by_jobs: BTreeMap<String, BTreeSet<String>>,
    blocks_jobs: BTreeMap<String, BTreeSet<String>>,
}

impl ConstraintMatrix {
    pub fn new(jobs: &[Job], constraints: &[(String, String)]) -> Result<Self, DagError> {
        let mut matrix = BTreeMap::<(String, String), Constraint>::new();
        let mut blocks_jobs = BTreeMap::new();
        let mut blocked_by_jobs = BTreeMap::new();

        for outer in jobs {
            for inner in jobs {
                let constraint = if outer.name == inner.name {
                    Constraint::Free
                } else {
                    Constraint::Indifferent
                };
                matrix.insert((outer.name.to_string(), inner.name.to_string()), constraint);
            }

            blocks_jobs.insert(outer.name.to_string(), BTreeSet::<String>::new());
            blocked_by_jobs.insert(outer.name.to_string(), BTreeSet::<String>::new());
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
                *cons = cons
                    .constrain()
                    .map_err(|_| DagError::CycleExistsBecauseOf(new_constraint.0.to_string()))?
            }
            if let Some(vec) = blocks_jobs.get_mut(&new_constraint.0) {
                vec.insert(new_constraint.1.to_string());
            }

            if let Some(vec) = blocked_by_jobs.get_mut(&new_constraint.1) {
                vec.insert(new_constraint.0.to_string());
            }
        }

        Ok(ConstraintMatrix {
            blocked_by_jobs,
            blocks_jobs,
        })
    }

    pub fn blocked_by(&self, link: &str) -> ConstraintMatrixConstraintIterator {
        ConstraintMatrixConstraintIterator::new(&self.blocks_jobs, link.to_string())
    }

    pub fn blocking(&self, link: &str) -> ConstraintMatrixConstraintIterator {
        ConstraintMatrixConstraintIterator::new(&self.blocked_by_jobs, link.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::job::dag::JobList;
    use crate::ci::job::tests::*;

    pub fn complex_matrix() -> Result<ConstraintMatrix, DagError> {
        let list = complex_job_schedule();
        ConstraintMatrix::new(&list.0, &list.1)
    }

    #[test]
    pub fn fails_on_unknown_job_in_constraint() {
        let jobs = vec![job("build")];

        let constraints = vec![cons("build1", "test1")];

        let matrix = ConstraintMatrix::new(&jobs, &constraints);
        assert!(matches!(matrix, Err(DagError::UnknownJobInConstraint(_))))
    }

    #[test]
    pub fn fails_on_unknown_bad_constraint() {
        let jobs = vec![job("build")];

        let constraints = vec![cons("build", "build")];

        let matrix = ConstraintMatrix::new(&jobs, &constraints);
        assert!(matches!(matrix, Err(DagError::JobCannotBlockItself(_))))
    }

    #[test]
    pub fn list_all_blocking() {
        let pipeline = complex_matrix().unwrap();
        let mut vec: Vec<String> = pipeline.blocking("deploy").collect();
        vec.sort();
        let list = JobList::from(vec);
        assert_eq!("[build1, build2, test1, test2]", format!("{}", list))
    }

    #[test]
    pub fn list_all_blocks() {
        let pipeline = complex_matrix().unwrap();
        let list = JobList::from(pipeline.blocked_by("test1").collect());
        assert_eq!("[deploy]", format!("{}", list))
    }
}
