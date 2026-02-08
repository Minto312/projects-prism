pub const GITHUB_GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

pub const VALIDATE_TOKEN_QUERY: &str = r#"
query {
    viewer {
        login
    }
}
"#;

pub const FETCH_ACCESSIBLE_PROJECTS_QUERY: &str = r#"
query($cursor: String) {
    viewer {
        projectsV2(first: 100, after: $cursor) {
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
                    __typename
                    ... on User {
                        login
                    }
                    ... on Organization {
                        login
                    }
                }
            }
        }
    }
}
"#;

pub const FETCH_PROJECT_DETAILS_QUERY: &str = r#"
query($projectId: ID!) {
    node(id: $projectId) {
        ... on ProjectV2 {
            id
            title
            url
            updatedAt
            owner {
                __typename
                ... on User {
                    login
                }
                ... on Organization {
                    login
                }
            }
            fields(first: 20) {
                nodes {
                    __typename
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
        }
    }
}
"#;

pub const FETCH_PROJECT_ITEMS_QUERY: &str = r#"
query($projectId: ID!, $cursor: String) {
    node(id: $projectId) {
        ... on ProjectV2 {
            items(first: 100, after: $cursor) {
                pageInfo {
                    hasNextPage
                    endCursor
                }
                nodes {
                    id
                    updatedAt
                    fieldValues(first: 20) {
                        nodes {
                            __typename
                            ... on ProjectV2ItemFieldSingleSelectValue {
                                field {
                                    ... on ProjectV2SingleSelectField {
                                        name
                                    }
                                }
                                optionId
                            }
                            ... on ProjectV2ItemFieldDateValue {
                                field {
                                    ... on ProjectV2Field {
                                        name
                                    }
                                }
                                date
                            }
                        }
                    }
                    content {
                        __typename
                        ... on Issue {
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
                        ... on PullRequest {
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
                        ... on DraftIssue {
                            title
                            body
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

pub const FETCH_ITEM_STATUS_QUERY: &str = r#"
query($itemId: ID!) {
    node(id: $itemId) {
        ... on ProjectV2Item {
            fieldValues(first: 20) {
                nodes {
                    __typename
                    ... on ProjectV2ItemFieldSingleSelectValue {
                        field {
                            ... on ProjectV2SingleSelectField {
                                id
                                name
                            }
                        }
                        optionId
                    }
                }
            }
        }
    }
}
"#;

pub const UPDATE_ITEM_STATUS_MUTATION: &str = r#"
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
    updateProjectV2ItemFieldValue(
        input: {
            projectId: $projectId
            itemId: $itemId
            fieldId: $fieldId
            value: { singleSelectOptionId: $optionId }
        }
    ) {
        projectV2Item {
            id
        }
    }
}
"#;

pub const RATE_LIMIT_QUERY: &str = r#"
query {
    rateLimit {
        limit
        remaining
        resetAt
    }
}
"#;
