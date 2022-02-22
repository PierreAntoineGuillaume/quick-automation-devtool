#[cfg(test)]
mod tests {
    use crate::ci::job::Job;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct ConstraintMatrix {
        map: HashMap<(String, String), Constraint>,
    }

    impl ConstraintMatrix {
        fn new(jobs: &[Job], constraints: &[(String, String)]) -> Result<Self, DagError> {
            let mut matrix = HashMap::<(String, String), Constraint>::new();
            for outer in jobs {
                for inner in jobs {
                    let constraint = if outer.name == inner.name {
                        Constraint::Free
                    } else {
                        Constraint::Indifferent
                    };
                    matrix.insert((outer.name.to_string(), inner.name.to_string()), constraint);
                }
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
            }

            Ok(ConstraintMatrix { map: matrix })
        }
    }

    #[derive(Debug)]
    enum Constraint {
        Free,
        Indifferent,
        Blocked(usize),
    }

    impl Constraint {
        fn constrain(&self) -> Result<Self, ()> {
            match self {
                Constraint::Free => Err(()),
                Constraint::Indifferent => Ok(Constraint::Blocked(1)),
                Constraint::Blocked(usize) => Ok(Constraint::Blocked(*usize + 1usize)),
            }
        }
    }

    impl Default for Constraint {
        fn default() -> Self {
            Self::Indifferent
        }
    }

    #[derive(Debug)]
    pub enum DagError {
        JobCannotBlockItself(String),
        UnknownJobInConstraint(String),
        CycleExistsBetween(String, String),
    }

    fn job(name: &str) -> Job {
        Job {
            name: name.to_string(),
            instructions: vec![],
        }
    }

    fn cons(blocking: &str, blocked: &str) -> (String, String) {
        (blocking.to_string(), blocked.to_string())
    }

    #[test]
    pub fn create() {
        let matrix = complex_pipeline();
        assert!(matrix.is_ok())
    }

    fn complex_pipeline() -> Result<ConstraintMatrix, DagError> {
        let jobs = vec![
            job("deploy"),
            job("build1"),
            job("build2"),
            job("test1"),
            job("test2"),
        ];

        let constraints = vec![
            cons("build1", "test1"),
            cons("build1", "test2"),
            cons("build2", "test1"),
            cons("build2", "test2"),
            cons("test1", "deploy"),
            cons("test2", "deploy"),
        ];

        ConstraintMatrix::new(&jobs, &constraints)
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
    pub fn check_direct_links() {
        let matrix = complex_pipeline().unwrap();
        assert!(matches!(
            matrix.map.get(&cons("build1", "build2")).unwrap(),
            Constraint::Indifferent
        ));
        assert!(matches!(
            matrix.map.get(&cons("build1", "test1")).unwrap(),
            Constraint::Blocked(1)
        ));
        assert!(matches!(
            matrix.map.get(&cons("build1", "build1")).unwrap(),
            Constraint::Free
        ))
    }
}
