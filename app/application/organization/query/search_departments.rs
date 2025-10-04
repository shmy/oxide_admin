use crate::organization::dto::department::DepartmentDto;
use crate::organization::dto::department::DepartmentWithChildren;
use crate::shared::query_handler::QueryHandler;
use bon::Builder;
use domain::organization::error::OrganizationError;
use infrastructure::shared::pg_pool::PgPool;
use nject::injectable;
use serde::Deserialize;
use serde_with::serde_as;
use single_flight::single_flight;
use std::collections::HashMap;
use utoipa::IntoParams;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, IntoParams, Builder)]
pub struct SearchDepartmentsQuery {}

#[derive(Debug, Clone)]
#[injectable]
pub struct SearchDepartmentsQueryHandler {
    pool: PgPool,
}

impl QueryHandler for SearchDepartmentsQueryHandler {
    type Query = SearchDepartmentsQuery;
    type Output = Vec<DepartmentWithChildren>;
    type Error = OrganizationError;

    #[single_flight]
    #[tracing::instrument]
    async fn query(
        &self,
        query: SearchDepartmentsQuery,
    ) -> Result<Vec<DepartmentWithChildren>, OrganizationError> {
        let departments = sqlx::query_as!(
            DepartmentDto,
            r#"
        SELECT id, name, code, parent_code
        FROM _departments
        "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Self::build_department_tree(departments))
    }
}

impl SearchDepartmentsQueryHandler {
    pub fn build_department_tree(departments: Vec<DepartmentDto>) -> Vec<DepartmentWithChildren> {
        // 先构建 code => Vec<子部门> 的映射
        let mut children_map: HashMap<Option<String>, Vec<DepartmentDto>> = HashMap::new();
        for dept in departments {
            children_map
                .entry(dept.parent_code.clone())
                .or_default()
                .push(dept);
        }

        // 递归构建节点
        fn build_node(
            dept: &DepartmentDto,
            children_map: &HashMap<Option<String>, Vec<DepartmentDto>>,
        ) -> DepartmentWithChildren {
            let children = children_map.get(&Some(dept.code.clone())).map(|children| {
                children
                    .iter()
                    .map(|child| build_node(child, children_map))
                    .collect()
            });

            DepartmentWithChildren {
                id: dept.id.clone(),
                label: dept.name.clone(),
                value: dept.code.clone(),
                children: children.unwrap_or_default(),
            }
        }

        // 构建所有 parent_code == None 的根节点
        children_map
            .get(&None)
            .unwrap_or(&vec![])
            .iter()
            .map(|dept| build_node(dept, &children_map))
            .collect()
    }
}
