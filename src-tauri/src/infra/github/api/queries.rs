/// 認証ユーザー情報の取得
pub const VIEWER_QUERY: &str = r#"
query {
  viewer {
    login
  }
}
"#;

/// ユーザーのプロジェクト一覧取得
pub const USER_PROJECTS_QUERY: &str = r#"
query($login: String!, $after: String) {
  user(login: $login) {
    projectsV2(first: 20, after: $after, orderBy: {field: UPDATED_AT, direction: DESC}) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        id
        title
        url
        updatedAt
      }
    }
  }
}
"#;

/// Organization のプロジェクト一覧取得
pub const ORG_PROJECTS_QUERY: &str = r#"
query($login: String!, $after: String) {
  organization(login: $login) {
    projectsV2(first: 20, after: $after, orderBy: {field: UPDATED_AT, direction: DESC}) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        id
        title
        url
        updatedAt
      }
    }
  }
}
"#;

/// viewer がアクセス可能な全プロジェクトの一覧（Organization含む）
pub const VIEWER_PROJECTS_QUERY: &str = r#"
query($after: String) {
  viewer {
    login
    projectsV2(first: 20, after: $after, orderBy: {field: UPDATED_AT, direction: DESC}) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        id
        title
        url
        updatedAt
        owner {
          ... on User {
            __typename
            login
          }
          ... on Organization {
            __typename
            login
          }
        }
      }
    }
  }
}
"#;

/// プロジェクトのアイテム一覧取得（Status フィールド情報含む）
pub const PROJECT_ITEMS_QUERY: &str = r#"
query($projectId: ID!, $after: String) {
  node(id: $projectId) {
    ... on ProjectV2 {
      id
      title
      url
      updatedAt
      owner {
        ... on User {
          __typename
          login
        }
        ... on Organization {
          __typename
          login
        }
      }
      fields(first: 20) {
        nodes {
          ... on ProjectV2SingleSelectField {
            id
            name
            options {
              id
              name
              color
            }
          }
        }
      }
      items(first: 100, after: $after, orderBy: {field: POSITION, direction: ASC}) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          id
          updatedAt
          fieldValueByName(name: "Status") {
            ... on ProjectV2ItemFieldSingleSelectValue {
              optionId
            }
          }
          content {
            ... on Issue {
              __typename
              id
              title
              body
              url
              assignees(first: 1) {
                nodes {
                  login
                }
              }
              labels(first: 10) {
                nodes {
                  name
                }
              }
            }
            ... on DraftIssue {
              __typename
              title
              body
              assignees(first: 1) {
                nodes {
                  login
                }
              }
            }
            ... on PullRequest {
              __typename
              id
              title
              body
              url
              assignees(first: 1) {
                nodes {
                  login
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;

/// 単一アイテムの Status 取得
pub const ITEM_STATUS_QUERY: &str = r#"
query($itemId: ID!, $projectId: ID!) {
  node(id: $itemId) {
    ... on ProjectV2Item {
      id
      updatedAt
      project {
        id
      }
      fieldValueByName(name: "Status") {
        ... on ProjectV2ItemFieldSingleSelectValue {
          optionId
        }
      }
    }
  }
}
"#;

/// Status フィールド値の更新
pub const UPDATE_ITEM_STATUS_MUTATION: &str = r#"
mutation($input: UpdateProjectV2ItemFieldValueInput!) {
  updateProjectV2ItemFieldValue(input: $input) {
    projectV2Item {
      id
      updatedAt
    }
  }
}
"#;

/// Rate Limit 情報取得
pub const RATE_LIMIT_QUERY: &str = r#"
query {
  rateLimit {
    limit
    cost
    remaining
    resetAt
  }
}
"#;
