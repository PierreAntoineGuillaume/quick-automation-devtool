#[derive(Default, Clone)]
pub struct JobDesc {
    pub name: String,
    pub script: Vec<String>,
    pub image: Option<String>,
    pub group: Option<String>,
}

#[derive(Default, Clone)]
pub struct CiConfig {
    pub jobs: Vec<JobDesc>,
    pub groups: Vec<String>,
    pub constraints: Vec<(String, String)>,
}
