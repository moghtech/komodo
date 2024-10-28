import * as Types from "./types.js";
export type AuthResponses = {
    GetLoginOptions: Types.GetLoginOptionsResponse;
    CreateLocalUser: Types.CreateLocalUserResponse;
    LoginLocalUser: Types.LoginLocalUserResponse;
    ExchangeForJwt: Types.ExchangeForJwtResponse;
    GetUser: Types.GetUserResponse;
};
export type UserResponses = {
    PushRecentlyViewed: Types.PushRecentlyViewedResponse;
    SetLastSeenUpdate: Types.SetLastSeenUpdateResponse;
    CreateApiKey: Types.CreateApiKeyResponse;
    DeleteApiKey: Types.DeleteApiKeyResponse;
};
export type ReadResponses = {
    GetVersion: Types.GetVersionResponse;
    GetCoreInfo: Types.GetCoreInfoResponse;
    ListSecrets: Types.ListSecretsResponse;
    ListGitProvidersFromConfig: Types.ListGitProvidersFromConfigResponse;
    ListDockerRegistriesFromConfig: Types.ListDockerRegistriesFromConfigResponse;
    GetUsername: Types.GetUsernameResponse;
    GetPermissionLevel: Types.GetPermissionLevelResponse;
    FindUser: Types.FindUserResponse;
    ListUsers: Types.ListUsersResponse;
    ListApiKeys: Types.ListApiKeysResponse;
    ListApiKeysForServiceUser: Types.ListApiKeysForServiceUserResponse;
    ListPermissions: Types.ListPermissionsResponse;
    ListUserTargetPermissions: Types.ListUserTargetPermissionsResponse;
    GetUserGroup: Types.GetUserGroupResponse;
    ListUserGroups: Types.ListUserGroupsResponse;
    FindResources: Types.FindResourcesResponse;
    GetProceduresSummary: Types.GetProceduresSummaryResponse;
    GetProcedure: Types.GetProcedureResponse;
    GetProcedureActionState: Types.GetProcedureActionStateResponse;
    ListProcedures: Types.ListProceduresResponse;
    ListFullProcedures: Types.ListFullProceduresResponse;
    GetActionsSummary: Types.GetActionsSummaryResponse;
    GetAction: Types.GetActionResponse;
    GetActionActionState: Types.GetActionActionStateResponse;
    ListActions: Types.ListActionsResponse;
    ListFullActions: Types.ListFullActionsResponse;
    GetServerTemplate: Types.GetServerTemplateResponse;
    GetServerTemplatesSummary: Types.GetServerTemplatesSummaryResponse;
    ListServerTemplates: Types.ListServerTemplatesResponse;
    ListFullServerTemplates: Types.ListFullServerTemplatesResponse;
    GetServersSummary: Types.GetServersSummaryResponse;
    GetServer: Types.GetServerResponse;
    GetServerState: Types.GetServerStateResponse;
    GetPeripheryVersion: Types.GetPeripheryVersionResponse;
    ListDockerContainers: Types.ListDockerContainersResponse;
    ListAllDockerContainers: Types.ListAllDockerContainersResponse;
    InspectDockerContainer: Types.InspectDockerContainerResponse;
    GetResourceMatchingContainer: Types.GetResourceMatchingContainerResponse;
    GetContainerLog: Types.GetContainerLogResponse;
    SearchContainerLog: Types.SearchContainerLogResponse;
    ListDockerNetworks: Types.ListDockerNetworksResponse;
    InspectDockerNetwork: Types.InspectDockerNetworkResponse;
    ListDockerImages: Types.ListDockerImagesResponse;
    InspectDockerImage: Types.InspectDockerImageResponse;
    ListDockerImageHistory: Types.ListDockerImageHistoryResponse;
    ListDockerVolumes: Types.ListDockerVolumesResponse;
    InspectDockerVolume: Types.InspectDockerVolumeResponse;
    ListComposeProjects: Types.ListComposeProjectsResponse;
    GetServerActionState: Types.GetServerActionStateResponse;
    GetHistoricalServerStats: Types.GetHistoricalServerStatsResponse;
    ListServers: Types.ListServersResponse;
    ListFullServers: Types.ListFullServersResponse;
    GetDeploymentsSummary: Types.GetDeploymentsSummaryResponse;
    GetDeployment: Types.GetDeploymentResponse;
    GetDeploymentContainer: Types.GetDeploymentContainerResponse;
    GetDeploymentActionState: Types.GetDeploymentActionStateResponse;
    GetDeploymentStats: Types.GetDeploymentStatsResponse;
    GetDeploymentLog: Types.GetDeploymentLogResponse;
    SearchDeploymentLog: Types.SearchDeploymentLogResponse;
    ListDeployments: Types.ListDeploymentsResponse;
    ListFullDeployments: Types.ListFullDeploymentsResponse;
    ListCommonDeploymentExtraArgs: Types.ListCommonDeploymentExtraArgsResponse;
    GetBuildsSummary: Types.GetBuildsSummaryResponse;
    GetBuild: Types.GetBuildResponse;
    GetBuildActionState: Types.GetBuildActionStateResponse;
    GetBuildMonthlyStats: Types.GetBuildMonthlyStatsResponse;
    GetBuildWebhookEnabled: Types.GetBuildWebhookEnabledResponse;
    ListBuilds: Types.ListBuildsResponse;
    ListFullBuilds: Types.ListFullBuildsResponse;
    ListBuildVersions: Types.ListBuildVersionsResponse;
    ListCommonBuildExtraArgs: Types.ListCommonBuildExtraArgsResponse;
    GetReposSummary: Types.GetReposSummaryResponse;
    GetRepo: Types.GetRepoResponse;
    GetRepoActionState: Types.GetRepoActionStateResponse;
    GetRepoWebhooksEnabled: Types.GetRepoWebhooksEnabledResponse;
    ListRepos: Types.ListReposResponse;
    ListFullRepos: Types.ListFullReposResponse;
    GetResourceSyncsSummary: Types.GetResourceSyncsSummaryResponse;
    GetResourceSync: Types.GetResourceSyncResponse;
    GetResourceSyncActionState: Types.GetResourceSyncActionStateResponse;
    GetSyncWebhooksEnabled: Types.GetSyncWebhooksEnabledResponse;
    ListResourceSyncs: Types.ListResourceSyncsResponse;
    ListFullResourceSyncs: Types.ListFullResourceSyncsResponse;
    GetStacksSummary: Types.GetStacksSummaryResponse;
    GetStack: Types.GetStackResponse;
    GetStackActionState: Types.GetStackActionStateResponse;
    GetStackWebhooksEnabled: Types.GetStackWebhooksEnabledResponse;
    GetStackServiceLog: Types.GetStackServiceLogResponse;
    SearchStackServiceLog: Types.SearchStackServiceLogResponse;
    ListStacks: Types.ListStacksResponse;
    ListFullStacks: Types.ListFullStacksResponse;
    ListStackServices: Types.ListStackServicesResponse;
    ListCommonStackExtraArgs: Types.ListCommonStackExtraArgsResponse;
    ListCommonStackBuildExtraArgs: Types.ListCommonStackBuildExtraArgsResponse;
    GetBuildersSummary: Types.GetBuildersSummaryResponse;
    GetBuilder: Types.GetBuilderResponse;
    ListBuilders: Types.ListBuildersResponse;
    ListFullBuilders: Types.ListFullBuildersResponse;
    GetAlertersSummary: Types.GetAlertersSummaryResponse;
    GetAlerter: Types.GetAlerterResponse;
    ListAlerters: Types.ListAlertersResponse;
    ListFullAlerters: Types.ListFullAlertersResponse;
    ExportAllResourcesToToml: Types.ExportAllResourcesToTomlResponse;
    ExportResourcesToToml: Types.ExportResourcesToTomlResponse;
    GetTag: Types.GetTagResponse;
    ListTags: Types.ListTagsResponse;
    GetUpdate: Types.GetUpdateResponse;
    ListUpdates: Types.ListUpdatesResponse;
    ListAlerts: Types.ListAlertsResponse;
    GetAlert: Types.GetAlertResponse;
    GetSystemInformation: Types.GetSystemInformationResponse;
    GetSystemStats: Types.GetSystemStatsResponse;
    ListSystemProcesses: Types.ListSystemProcessesResponse;
    GetVariable: Types.GetVariableResponse;
    ListVariables: Types.ListVariablesResponse;
    GetGitProviderAccount: Types.GetGitProviderAccountResponse;
    ListGitProviderAccounts: Types.ListGitProviderAccountsResponse;
    GetDockerRegistryAccount: Types.GetDockerRegistryAccountResponse;
    ListDockerRegistryAccounts: Types.ListDockerRegistryAccountsResponse;
};
export type WriteResponses = {
    UpdateUserUsername: Types.UpdateUserUsername;
    UpdateUserPassword: Types.UpdateUserPassword;
    DeleteUser: Types.DeleteUser;
    CreateServiceUser: Types.CreateServiceUserResponse;
    UpdateServiceUserDescription: Types.UpdateServiceUserDescriptionResponse;
    CreateApiKeyForServiceUser: Types.CreateApiKeyForServiceUserResponse;
    DeleteApiKeyForServiceUser: Types.DeleteApiKeyForServiceUserResponse;
    CreateUserGroup: Types.UserGroup;
    RenameUserGroup: Types.UserGroup;
    DeleteUserGroup: Types.UserGroup;
    AddUserToUserGroup: Types.UserGroup;
    RemoveUserFromUserGroup: Types.UserGroup;
    SetUsersInUserGroup: Types.UserGroup;
    UpdateUserAdmin: Types.UpdateUserAdminResponse;
    UpdateUserBasePermissions: Types.UpdateUserBasePermissionsResponse;
    UpdatePermissionOnResourceType: Types.UpdatePermissionOnResourceTypeResponse;
    UpdatePermissionOnTarget: Types.UpdatePermissionOnTargetResponse;
    UpdateDescription: Types.UpdateDescriptionResponse;
    LaunchServer: Types.Update;
    CreateServer: Types.Server;
    DeleteServer: Types.Server;
    UpdateServer: Types.Server;
    RenameServer: Types.Update;
    CreateNetwork: Types.Update;
    CreateDeployment: Types.Deployment;
    CopyDeployment: Types.Deployment;
    DeleteDeployment: Types.Deployment;
    UpdateDeployment: Types.Deployment;
    RenameDeployment: Types.Update;
    CreateBuild: Types.Build;
    CopyBuild: Types.Build;
    DeleteBuild: Types.Build;
    UpdateBuild: Types.Build;
    RenameBuild: Types.Update;
    RefreshBuildCache: Types.NoData;
    CreateBuildWebhook: Types.CreateBuildWebhookResponse;
    DeleteBuildWebhook: Types.DeleteBuildWebhookResponse;
    CreateBuilder: Types.Builder;
    CopyBuilder: Types.Builder;
    DeleteBuilder: Types.Builder;
    UpdateBuilder: Types.Builder;
    RenameBuilder: Types.Update;
    CreateServerTemplate: Types.ServerTemplate;
    CopyServerTemplate: Types.ServerTemplate;
    DeleteServerTemplate: Types.ServerTemplate;
    UpdateServerTemplate: Types.ServerTemplate;
    RenameServerTemplate: Types.Update;
    CreateRepo: Types.Repo;
    CopyRepo: Types.Repo;
    DeleteRepo: Types.Repo;
    UpdateRepo: Types.Repo;
    RenameRepo: Types.Update;
    RefreshRepoCache: Types.NoData;
    CreateRepoWebhook: Types.CreateRepoWebhookResponse;
    DeleteRepoWebhook: Types.DeleteRepoWebhookResponse;
    CreateAlerter: Types.Alerter;
    CopyAlerter: Types.Alerter;
    DeleteAlerter: Types.Alerter;
    UpdateAlerter: Types.Alerter;
    RenameAlerter: Types.Update;
    CreateProcedure: Types.Procedure;
    CopyProcedure: Types.Procedure;
    DeleteProcedure: Types.Procedure;
    UpdateProcedure: Types.Procedure;
    RenameProcedure: Types.Update;
    CreateAction: Types.Action;
    CopyAction: Types.Action;
    DeleteAction: Types.Action;
    UpdateAction: Types.Action;
    RenameAction: Types.Update;
    CreateResourceSync: Types.ResourceSync;
    CopyResourceSync: Types.ResourceSync;
    DeleteResourceSync: Types.ResourceSync;
    UpdateResourceSync: Types.ResourceSync;
    RenameResourceSync: Types.Update;
    CommitSync: Types.ResourceSync;
    WriteSyncFileContents: Types.Update;
    RefreshResourceSyncPending: Types.ResourceSync;
    CreateSyncWebhook: Types.CreateSyncWebhookResponse;
    DeleteSyncWebhook: Types.DeleteSyncWebhookResponse;
    CreateStack: Types.Stack;
    CopyStack: Types.Stack;
    DeleteStack: Types.Stack;
    UpdateStack: Types.Stack;
    RenameStack: Types.Update;
    WriteStackFileContents: Types.Update;
    RefreshStackCache: Types.NoData;
    CreateStackWebhook: Types.CreateStackWebhookResponse;
    DeleteStackWebhook: Types.DeleteStackWebhookResponse;
    CreateTag: Types.Tag;
    DeleteTag: Types.Tag;
    RenameTag: Types.Tag;
    UpdateTagsOnResource: Types.UpdateTagsOnResourceResponse;
    CreateVariable: Types.CreateVariableResponse;
    UpdateVariableValue: Types.UpdateVariableValueResponse;
    UpdateVariableDescription: Types.UpdateVariableDescriptionResponse;
    UpdateVariableIsSecret: Types.UpdateVariableIsSecretResponse;
    DeleteVariable: Types.DeleteVariableResponse;
    CreateGitProviderAccount: Types.CreateGitProviderAccountResponse;
    UpdateGitProviderAccount: Types.UpdateGitProviderAccountResponse;
    DeleteGitProviderAccount: Types.DeleteGitProviderAccountResponse;
    CreateDockerRegistryAccount: Types.CreateDockerRegistryAccountResponse;
    UpdateDockerRegistryAccount: Types.UpdateDockerRegistryAccountResponse;
    DeleteDockerRegistryAccount: Types.DeleteDockerRegistryAccountResponse;
};
export type ExecuteResponses = {
    StartContainer: Types.Update;
    RestartContainer: Types.Update;
    PauseContainer: Types.Update;
    UnpauseContainer: Types.Update;
    StopContainer: Types.Update;
    DestroyContainer: Types.Update;
    StartAllContainers: Types.Update;
    RestartAllContainers: Types.Update;
    PauseAllContainers: Types.Update;
    UnpauseAllContainers: Types.Update;
    StopAllContainers: Types.Update;
    PruneContainers: Types.Update;
    DeleteNetwork: Types.Update;
    PruneNetworks: Types.Update;
    DeleteImage: Types.Update;
    PruneImages: Types.Update;
    DeleteVolume: Types.Update;
    PruneVolumes: Types.Update;
    PruneDockerBuilders: Types.Update;
    PruneBuildx: Types.Update;
    PruneSystem: Types.Update;
    Deploy: Types.Update;
    StartDeployment: Types.Update;
    RestartDeployment: Types.Update;
    PauseDeployment: Types.Update;
    UnpauseDeployment: Types.Update;
    StopDeployment: Types.Update;
    DestroyDeployment: Types.Update;
    RunBuild: Types.Update;
    CancelBuild: Types.Update;
    CloneRepo: Types.Update;
    PullRepo: Types.Update;
    BuildRepo: Types.Update;
    CancelRepoBuild: Types.Update;
    RunProcedure: Types.Update;
    RunAction: Types.Update;
    BatchRunAction: Types.BatchExecutionResult;
    LaunchServer: Types.Update;
    RunSync: Types.Update;
    DeployStack: Types.Update;
    DeployStackIfChanged: Types.Update;
    StartStack: Types.Update;
    RestartStack: Types.Update;
    StopStack: Types.Update;
    PauseStack: Types.Update;
    UnpauseStack: Types.Update;
    DestroyStack: Types.Update;
    DeployStackService: Types.Update;
    StartStackService: Types.Update;
    RestartStackService: Types.Update;
    StopStackService: Types.Update;
    PauseStackService: Types.Update;
    UnpauseStackService: Types.Update;
    DestroyStackService: Types.Update;
};
