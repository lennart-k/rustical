interface Window {
  rusticalUser: {
    id: String,
    displayname: String | null,
    memberships: Array<String>,
    principal_type: "individual" | "group" | "room" | String
  }
}

