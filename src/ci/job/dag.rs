mod constraint_matrix;
mod constraint_matrix_constraint_iterator;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::job::dag::constraint_matrix::ConstraintMatrix;
    use crate::ci::job::Job;

    pub fn job(name: &str) -> Job {
        Job {
            name: name.to_string(),
            instructions: vec![],
        }
    }

    pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
        (blocking.to_string(), blocked.to_string())
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

    pub fn complex_pipeline() -> Result<ConstraintMatrix, DagError> {
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
    pub fn list_all_blocking() {
        let pipeline = complex_pipeline().unwrap();
        let vector: Vec<String> = pipeline.blocking("deploy").collect();
        assert_eq!(
            r#"["build1", "build2", "test1", "test2"]"#,
            format!("{:?}", vector)
        )
    }

    #[test]
    pub fn list_all_blocks() {
        let pipeline = complex_pipeline().unwrap();
        let vector: Vec<String> = pipeline.blocked_by("test1").collect();
        assert_eq!(r#"["deploy"]"#, format!("{:?}", vector))
    }
}
