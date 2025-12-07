import * as Types from "./types.js";

export type AuthResponses = {
  GetLoginOptions: Types.GetLoginOptionsResponse;
  SignUpLocalUser: Types.SignUpLocalUserResponse;
  LoginLocalUser: Types.LoginLocalUserResponse;
  ExchangeForJwt: Types.ExchangeForJwtResponse;
  CompleteTotpLogin: Types.CompleteTotpLoginResponse;
  GetUser: Types.GetUserResponse;
};

export type UserResponses = {
  PushRecentlyViewed: Types.PushRecentlyViewedResponse;
  SetLastSeenUpdate: Types.SetLastSeenUpdateResponse;
  CreateApiKey: Types.CreateApiKeyResponse;
  DeleteApiKey: Types.DeleteApiKeyResponse;
  BeginTotpEnrollment: Types.BeginTotpEnrollmentResponse;
  ConfirmTotpEnrollment: Types.ConfirmTotpEnrollmentResponse;
};

export type ReadResponses = {
  GetVersion: Types.GetVersionResponse;
  GetCoreInfo: Types.GetCoreInfoResponse;
  ListSecrets: Types.ListSecretsResponse;
  ListGitProvidersFromConfig: Types.ListGitProvidersFromConfigResponse;
  ListDockerRegistriesFromConfig: Types.ListDockerRegistriesFromConfigResponse;

  // ==== SWARM ====
  GetSwarmsSummary: Types.GetSwarmsSummaryResponse;
  GetSwarm: Types.GetSwarmResponse;
  GetSwarmActionState: Types.GetSwarmActionStateResponse;
  ListSwarms: Types.ListSwarmsResponse;
  ListFullSwarms: Types.ListFullSwarmsResponse;
  InspectSwarm: Types.InspectSwarmResponse;
  ListSwarmNodes: Types.ListSwarmNodesResponse;
  InspectSwarmNode: Types.InspectSwarmNodeResponse;
  ListSwarmConfigs: Types.ListSwarmConfigsResponse;
  InspectSwarmConfig: Types.InspectSwarmConfigResponse;
  ListSwarmSecrets: Types.ListSwarmSecretsResponse;
  InspectSwarmSecret: Types.InspectSwarmSecretResponse;
  ListSwarmStacks: Types.ListSwarmStacksResponse;
  InspectSwarmStack: Types.InspectSwarmStackResponse;
  ListSwarmTasks: Types.ListSwarmTasksResponse;
  InspectSwarmTask: Types.InspectSwarmTaskResponse;
  ListSwarmServices: Types.ListSwarmServicesResponse;
  InspectSwarmService: Types.InspectSwarmServiceResponse;
  GetSwarmServiceLog: Types.GetSwarmServiceLogResponse;
  SearchSwarmServiceLog: Types.SearchSwarmServiceLogResponse;
  ListSwarmNetworks: Types.ListSwarmNetworksResponse;

  // ==== SERVER ====
  GetServersSummary: Types.GetServersSummaryResponse;
  GetServer: Types.GetServerResponse;
  GetServerState: Types.GetServerStateResponse;
  GetPeripheryInformation: Types.GetPeripheryInformationResponse;
  GetServerActionState: Types.GetServerActionStateResponse;
  ListServers: Types.ListServersResponse;
  ListFullServers: Types.ListFullServersResponse;

  // ==== TERMINAL ====
  ListTerminals: Types.ListTerminalsResponse;

  // ==== DOCKER ====
  GetDockerContainersSummary: Types.GetDockerContainersSummaryResponse;
  ListAllDockerContainers: Types.ListAllDockerContainersResponse;
  ListDockerContainers: Types.ListDockerContainersResponse;
  InspectDockerContainer: Types.InspectDockerContainerResponse;
  GetResourceMatchingContainer: Types.GetResourceMatchingContainerResponse;
  GetContainerLog: Types.GetContainerLogResponse;
  SearchContainerLog: Types.SearchContainerLogResponse;
  ListComposeProjects: Types.ListComposeProjectsResponse;
  ListDockerNetworks: Types.ListDockerNetworksResponse;
  InspectDockerNetwork: Types.InspectDockerNetworkResponse;
  ListDockerImages: Types.ListDockerImagesResponse;
  InspectDockerImage: Types.InspectDockerImageResponse;
  ListDockerImageHistory: Types.ListDockerImageHistoryResponse;
  ListDockerVolumes: Types.ListDockerVolumesResponse;
  InspectDockerVolume: Types.InspectDockerVolumeResponse;

  // ==== SERVER STATS ====
  GetSystemInformation: Types.GetSystemInformationResponse;
  GetSystemStats: Types.GetSystemStatsResponse;
  GetHistoricalServerStats: Types.GetHistoricalServerStatsResponse;
  ListSystemProcesses: Types.ListSystemProcessesResponse;

  // ==== STACK ====
  GetStacksSummary: Types.GetStacksSummaryResponse;
  GetStack: Types.GetStackResponse;
  GetStackActionState: Types.GetStackActionStateResponse;
  GetStackLog: Types.GetStackLogResponse;
  SearchStackLog: Types.SearchStackLogResponse;
  InspectStackContainer: Types.InspectStackContainerResponse;
  InspectStackSwarmService: Types.InspectStackSwarmServiceResponse;
  ListStacks: Types.ListStacksResponse;
  ListFullStacks: Types.ListFullStacksResponse;
  ListStackServices: Types.ListStackServicesResponse;
  ListCommonStackExtraArgs: Types.ListCommonStackExtraArgsResponse;
  ListCommonStackBuildExtraArgs: Types.ListCommonStackBuildExtraArgsResponse;

  // ==== DEPLOYMENT ====
  GetDeploymentsSummary: Types.GetDeploymentsSummaryResponse;
  GetDeployment: Types.GetDeploymentResponse;
  GetDeploymentContainer: Types.GetDeploymentContainerResponse;
  GetDeploymentActionState: Types.GetDeploymentActionStateResponse;
  GetDeploymentStats: Types.GetDeploymentStatsResponse;
  GetDeploymentLog: Types.GetDeploymentLogResponse;
  SearchDeploymentLog: Types.SearchDeploymentLogResponse;
  InspectDeploymentContainer: Types.InspectDeploymentContainerResponse;
  InspectDeploymentSwarmService: Types.InspectDeploymentSwarmServiceResponse;
  ListDeployments: Types.ListDeploymentsResponse;
  ListFullDeployments: Types.ListFullDeploymentsResponse;
  ListCommonDeploymentExtraArgs: Types.ListCommonDeploymentExtraArgsResponse;

  // ==== BUILD ====
  GetBuildsSummary: Types.GetBuildsSummaryResponse;
  GetBuild: Types.GetBuildResponse;
  GetBuildActionState: Types.GetBuildActionStateResponse;
  GetBuildMonthlyStats: Types.GetBuildMonthlyStatsResponse;
  ListBuilds: Types.ListBuildsResponse;
  ListFullBuilds: Types.ListFullBuildsResponse;
  ListBuildVersions: Types.ListBuildVersionsResponse;
  ListCommonBuildExtraArgs: Types.ListCommonBuildExtraArgsResponse;

  // ==== REPO ====
  GetReposSummary: Types.GetReposSummaryResponse;
  GetRepo: Types.GetRepoResponse;
  GetRepoActionState: Types.GetRepoActionStateResponse;
  ListRepos: Types.ListReposResponse;
  ListFullRepos: Types.ListFullReposResponse;

  // ==== PROCEDURE ====
  GetProceduresSummary: Types.GetProceduresSummaryResponse;
  GetProcedure: Types.GetProcedureResponse;
  GetProcedureActionState: Types.GetProcedureActionStateResponse;
  ListProcedures: Types.ListProceduresResponse;
  ListFullProcedures: Types.ListFullProceduresResponse;

  // ==== ACTION ====
  GetActionsSummary: Types.GetActionsSummaryResponse;
  GetAction: Types.GetActionResponse;
  GetActionActionState: Types.GetActionActionStateResponse;
  ListActions: Types.ListActionsResponse;
  ListFullActions: Types.ListFullActionsResponse;

  // ==== SCHEDULE ====
  ListSchedules: Types.ListSchedulesResponse;

  // ==== SYNC ====
  GetResourceSyncsSummary: Types.GetResourceSyncsSummaryResponse;
  GetResourceSync: Types.GetResourceSyncResponse;
  GetResourceSyncActionState: Types.GetResourceSyncActionStateResponse;
  ListResourceSyncs: Types.ListResourceSyncsResponse;
  ListFullResourceSyncs: Types.ListFullResourceSyncsResponse;

  // ==== BUILDER ====
  GetBuildersSummary: Types.GetBuildersSummaryResponse;
  GetBuilder: Types.GetBuilderResponse;
  ListBuilders: Types.ListBuildersResponse;
  ListFullBuilders: Types.ListFullBuildersResponse;

  // ==== ALERTER ====
  GetAlertersSummary: Types.GetAlertersSummaryResponse;
  GetAlerter: Types.GetAlerterResponse;
  ListAlerters: Types.ListAlertersResponse;
  ListFullAlerters: Types.ListFullAlertersResponse;

  // ==== TOML ====
  ExportAllResourcesToToml: Types.ExportAllResourcesToTomlResponse;
  ExportResourcesToToml: Types.ExportResourcesToTomlResponse;

  // ==== TAG ====
  GetTag: Types.GetTagResponse;
  ListTags: Types.ListTagsResponse;

  // ==== USER ====
  GetUsername: Types.GetUsernameResponse;
  GetPermission: Types.GetPermissionResponse;
  FindUser: Types.FindUserResponse;
  ListUsers: Types.ListUsersResponse;
  ListApiKeys: Types.ListApiKeysResponse;
  ListApiKeysForServiceUser: Types.ListApiKeysForServiceUserResponse;
  ListPermissions: Types.ListPermissionsResponse;
  ListUserTargetPermissions: Types.ListUserTargetPermissionsResponse;

  // ==== USER GROUP ====
  GetUserGroup: Types.GetUserGroupResponse;
  ListUserGroups: Types.ListUserGroupsResponse;

  // ==== UPDATE ====
  GetUpdate: Types.GetUpdateResponse;
  ListUpdates: Types.ListUpdatesResponse;

  // ==== ALERT ====
  ListAlerts: Types.ListAlertsResponse;
  GetAlert: Types.GetAlertResponse;

  // ==== VARIABLE ====
  GetVariable: Types.GetVariableResponse;
  ListVariables: Types.ListVariablesResponse;

  // ==== PROVIDER ====
  GetGitProviderAccount: Types.GetGitProviderAccountResponse;
  ListGitProviderAccounts: Types.ListGitProviderAccountsResponse;
  GetDockerRegistryAccount: Types.GetDockerRegistryAccountResponse;
  ListDockerRegistryAccounts: Types.ListDockerRegistryAccountsResponse;

  // ==== ONBOARDING KEY ====
  ListOnboardingKeys: Types.ListOnboardingKeysResponse;
};

export type WriteResponses = {
  // ==== RESOURCE ====
  UpdateResourceMeta: Types.UpdateResourceMetaResponse;

  // ==== SWARM ====
  CreateSwarm: Types.Swarm;
  CopySwarm: Types.Swarm;
  DeleteSwarm: Types.Swarm;
  UpdateSwarm: Types.Swarm;
  RenameSwarm: Types.Update;

  // ==== SERVER ====
  CreateServer: Types.Server;
  CopyServer: Types.Server;
  DeleteServer: Types.Server;
  UpdateServer: Types.Server;
  RenameServer: Types.Update;
  CreateNetwork: Types.Update;
  UpdateServerPublicKey: Types.Update;
  RotateServerKeys: Types.Update;

  // ==== TERMINAL ====
  CreateTerminal: Types.NoData;
  DeleteTerminal: Types.NoData;
  DeleteAllTerminals: Types.NoData;
  BatchDeleteAllTerminals: Types.NoData;

  // ==== STACK ====
  CreateStack: Types.Stack;
  CopyStack: Types.Stack;
  DeleteStack: Types.Stack;
  UpdateStack: Types.Stack;
  RenameStack: Types.Update;
  WriteStackFileContents: Types.Update;
  RefreshStackCache: Types.NoData;

  // ==== DEPLOYMENT ====
  CreateDeployment: Types.Deployment;
  CopyDeployment: Types.Deployment;
  CreateDeploymentFromContainer: Types.Deployment;
  DeleteDeployment: Types.Deployment;
  UpdateDeployment: Types.Deployment;
  RenameDeployment: Types.Update;

  // ==== BUILD ====
  CreateBuild: Types.Build;
  CopyBuild: Types.Build;
  DeleteBuild: Types.Build;
  UpdateBuild: Types.Build;
  RenameBuild: Types.Update;
  WriteBuildFileContents: Types.Update;
  RefreshBuildCache: Types.NoData;

  // ==== REPO ====
  CreateRepo: Types.Repo;
  CopyRepo: Types.Repo;
  DeleteRepo: Types.Repo;
  UpdateRepo: Types.Repo;
  RenameRepo: Types.Update;
  RefreshRepoCache: Types.NoData;

  // ==== PROCEDURE ====
  CreateProcedure: Types.Procedure;
  CopyProcedure: Types.Procedure;
  DeleteProcedure: Types.Procedure;
  UpdateProcedure: Types.Procedure;
  RenameProcedure: Types.Update;

  // ==== ACTION ====
  CreateAction: Types.Action;
  CopyAction: Types.Action;
  DeleteAction: Types.Action;
  UpdateAction: Types.Action;
  RenameAction: Types.Update;

  // ==== SYNC ====
  CreateResourceSync: Types.ResourceSync;
  CopyResourceSync: Types.ResourceSync;
  DeleteResourceSync: Types.ResourceSync;
  UpdateResourceSync: Types.ResourceSync;
  RenameResourceSync: Types.Update;
  CommitSync: Types.Update;
  WriteSyncFileContents: Types.Update;
  RefreshResourceSyncPending: Types.ResourceSync;

  // ==== BUILDER ====
  CreateBuilder: Types.Builder;
  CopyBuilder: Types.Builder;
  DeleteBuilder: Types.Builder;
  UpdateBuilder: Types.Builder;
  RenameBuilder: Types.Update;

  // ==== ALERTER ====
  CreateAlerter: Types.Alerter;
  CopyAlerter: Types.Alerter;
  DeleteAlerter: Types.Alerter;
  UpdateAlerter: Types.Alerter;
  RenameAlerter: Types.Update;

  // ==== ONBOARDING KEY ====
  CreateOnboardingKey: Types.CreateOnboardingKeyResponse;
  UpdateOnboardingKey: Types.UpdateOnboardingKeyResponse;
  DeleteOnboardingKey: Types.DeleteOnboardingKeyResponse;

  // ==== USER ====
  CreateLocalUser: Types.CreateLocalUserResponse;
  UpdateUserUsername: Types.UpdateUserUsernameResponse;
  UpdateUserPassword: Types.UpdateUserPasswordResponse;
  DeleteUser: Types.DeleteUserResponse;

  // ==== SERVICE USER ====
  CreateServiceUser: Types.CreateServiceUserResponse;
  UpdateServiceUserDescription: Types.UpdateServiceUserDescriptionResponse;
  CreateApiKeyForServiceUser: Types.CreateApiKeyForServiceUserResponse;
  DeleteApiKeyForServiceUser: Types.DeleteApiKeyForServiceUserResponse;

  // ==== USER GROUP ====
  CreateUserGroup: Types.UserGroup;
  RenameUserGroup: Types.UserGroup;
  DeleteUserGroup: Types.UserGroup;
  AddUserToUserGroup: Types.UserGroup;
  RemoveUserFromUserGroup: Types.UserGroup;
  SetUsersInUserGroup: Types.UserGroup;
  SetEveryoneUserGroup: Types.UserGroup;

  // ==== PERMISSIONS ====
  UpdateUserAdmin: Types.UpdateUserAdminResponse;
  UpdateUserBasePermissions: Types.UpdateUserBasePermissionsResponse;
  UpdatePermissionOnResourceType: Types.UpdatePermissionOnResourceTypeResponse;
  UpdatePermissionOnTarget: Types.UpdatePermissionOnTargetResponse;

  // ==== TAG ====
  CreateTag: Types.Tag;
  DeleteTag: Types.Tag;
  RenameTag: Types.Tag;
  UpdateTagColor: Types.Tag;

  // ==== VARIABLE ====
  CreateVariable: Types.CreateVariableResponse;
  UpdateVariableValue: Types.UpdateVariableValueResponse;
  UpdateVariableDescription: Types.UpdateVariableDescriptionResponse;
  UpdateVariableIsSecret: Types.UpdateVariableIsSecretResponse;
  DeleteVariable: Types.DeleteVariableResponse;

  // ==== PROVIDER ====
  CreateGitProviderAccount: Types.CreateGitProviderAccountResponse;
  UpdateGitProviderAccount: Types.UpdateGitProviderAccountResponse;
  DeleteGitProviderAccount: Types.DeleteGitProviderAccountResponse;
  CreateDockerRegistryAccount: Types.CreateDockerRegistryAccountResponse;
  UpdateDockerRegistryAccount: Types.UpdateDockerRegistryAccountResponse;
  DeleteDockerRegistryAccount: Types.DeleteDockerRegistryAccountResponse;

  // ==== ALERT ====
  CloseAlert: Types.NoData;
};

export type ExecuteResponses = {
  // ==== STACK ====
  DeployStack: Types.Update;
  BatchDeployStack: Types.BatchExecutionResponse;
  DeployStackIfChanged: Types.Update;
  BatchDeployStackIfChanged: Types.BatchExecutionResponse;
  PullStack: Types.Update;
  BatchPullStack: Types.BatchExecutionResponse;
  StartStack: Types.Update;
  RestartStack: Types.Update;
  StopStack: Types.Update;
  PauseStack: Types.Update;
  UnpauseStack: Types.Update;
  DestroyStack: Types.Update;
  BatchDestroyStack: Types.BatchExecutionResponse;
  RunStackService: Types.Update;

  // ==== DEPLOYMENT ====
  Deploy: Types.Update;
  BatchDeploy: Types.BatchExecutionResponse;
  PullDeployment: Types.Update;
  StartDeployment: Types.Update;
  RestartDeployment: Types.Update;
  PauseDeployment: Types.Update;
  UnpauseDeployment: Types.Update;
  StopDeployment: Types.Update;
  DestroyDeployment: Types.Update;
  BatchDestroyDeployment: Types.BatchExecutionResponse;

  // ==== BUILD ====
  RunBuild: Types.Update;
  BatchRunBuild: Types.BatchExecutionResponse;
  CancelBuild: Types.Update;

  // ==== REPO ====
  CloneRepo: Types.Update;
  BatchCloneRepo: Types.BatchExecutionResponse;
  PullRepo: Types.Update;
  BatchPullRepo: Types.BatchExecutionResponse;
  BuildRepo: Types.Update;
  BatchBuildRepo: Types.BatchExecutionResponse;
  CancelRepoBuild: Types.Update;

  // ==== PROCEDURE ====
  RunProcedure: Types.Update;
  BatchRunProcedure: Types.BatchExecutionResponse;

  // ==== ACTION ====
  RunAction: Types.Update;
  BatchRunAction: Types.BatchExecutionResponse;

  // ==== SYNC ====
  RunSync: Types.Update;

  // ==== ALERTER ====
  TestAlerter: Types.Update;
  SendAlert: Types.Update;

  // ==== SERVER ====
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

  // ==== SWARM ====
  RemoveSwarmNodes: Types.Update;
  RemoveSwarmStacks: Types.Update;
  RemoveSwarmServices: Types.Update;
  RemoveSwarmConfigs: Types.Update;
  RemoveSwarmSecrets: Types.Update;

  // ==== MAINTENANCE ====
  ClearRepoCache: Types.Update;
  BackupCoreDatabase: Types.Update;
  GlobalAutoUpdate: Types.Update;
  RotateAllServerKeys: Types.Update;
  RotateCoreKeys: Types.Update;
};
