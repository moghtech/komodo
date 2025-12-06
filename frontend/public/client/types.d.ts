export interface MongoIdObj {
    $oid: string;
}
export type MongoId = MongoIdObj;
/** The levels of permission that a User or UserGroup can have on a resource. */
export declare enum PermissionLevel {
    /** No permissions. */
    None = "None",
    /** Can read resource information and config */
    Read = "Read",
    /** Can execute actions on the resource */
    Execute = "Execute",
    /** Can update the resource configuration */
    Write = "Write"
}
export interface PermissionLevelAndSpecifics {
    level: PermissionLevel;
    specific: Array<SpecificPermission>;
}
export type I64 = number;
export interface Resource<Config, Info> {
    /**
     * The Mongo ID of the resource.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized Resource<T>) }`
     */
    _id?: MongoId;
    /**
     * The resource name.
     * This is guaranteed unique among others of the same resource type.
     */
    name: string;
    /** A description for the resource */
    description?: string;
    /** Mark resource as a template */
    template?: boolean;
    /** Tag Ids */
    tags?: string[];
    /** Resource-specific information (not user configurable). */
    info?: Info;
    /** Resource-specific configuration. */
    config?: Config;
    /**
     * Set a base permission level that all users will have on the
     * resource.
     */
    base_permission?: PermissionLevelAndSpecifics | PermissionLevel;
    /** When description last updated */
    updated_at?: I64;
}
export declare enum ScheduleFormat {
    English = "English",
    Cron = "Cron"
}
export declare enum FileFormat {
    KeyValue = "key_value",
    Toml = "toml",
    Yaml = "yaml",
    Json = "json"
}
export interface ActionConfig {
    /** Whether this action should run at startup. */
    run_at_startup: boolean;
    /** Choose whether to specify schedule as regular CRON, or using the english to CRON parser. */
    schedule_format?: ScheduleFormat;
    /**
     * Optionally provide a schedule for the procedure to run on.
     *
     * There are 2 ways to specify a schedule:
     *
     * 1. Regular CRON expression:
     *
     * (second, minute, hour, day, month, day-of-week)
     * ```text
     * 0 0 0 1,15 * ?
     * ```
     *
     * 2. "English" expression via [english-to-cron](https://crates.io/crates/english-to-cron):
     *
     * ```text
     * at midnight on the 1st and 15th of the month
     * ```
     */
    schedule?: string;
    /**
     * Whether schedule is enabled if one is provided.
     * Can be used to temporarily disable the schedule.
     */
    schedule_enabled: boolean;
    /**
     * Optional. A TZ Identifier. If not provided, will use Core local timezone.
     * https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
     */
    schedule_timezone?: string;
    /** Whether to send alerts when the schedule was run. */
    schedule_alert: boolean;
    /** Whether to send alerts when this action fails. */
    failure_alert: boolean;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this procedure.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
    /**
     * Whether deno will be instructed to reload all dependencies,
     * this can usually be kept false outside of development.
     */
    reload_deno_deps?: boolean;
    /**
     * Typescript file contents using pre-initialized `komodo` client.
     * Supports variable / secret interpolation.
     */
    file_contents?: string;
    /**
     * Specify the format in which the arguments are defined.
     * Default: `key_value` (like environment)
     */
    arguments_format?: FileFormat;
    /** Default arguments to give to the Action for use in the script at `ARGS`. */
    arguments?: string;
}
/** Represents an empty json object: `{}` */
export interface NoData {
}
export type Action = Resource<ActionConfig, NoData>;
export interface ResourceListItem<Info> {
    /** The resource id */
    id: string;
    /** The resource type, ie `Server` or `Deployment` */
    type: ResourceTarget["type"];
    /** The resource name */
    name: string;
    /** Whether resource is a template */
    template: boolean;
    /** Tag Ids */
    tags: string[];
    /** Resource specific info */
    info: Info;
}
export declare enum ActionState {
    /** Unknown case */
    Unknown = "Unknown",
    /** Last clone / pull successful (or never cloned) */
    Ok = "Ok",
    /** Last clone / pull failed */
    Failed = "Failed",
    /** Currently running */
    Running = "Running"
}
export interface ActionListItemInfo {
    /** Whether last action run successful */
    state: ActionState;
    /** Action last successful run timestamp in ms. */
    last_run_at?: I64;
    /**
     * If the action has schedule enabled, this is the
     * next scheduled run time in unix ms.
     */
    next_scheduled_run?: I64;
    /**
     * If there is an error parsing schedule expression,
     * it will be given here.
     */
    schedule_error?: string;
}
export type ActionListItem = ResourceListItem<ActionListItemInfo>;
export declare enum TemplatesQueryBehavior {
    /** Include templates in results. Default. */
    Include = "Include",
    /** Exclude templates from results. */
    Exclude = "Exclude",
    /** Results *only* includes templates. */
    Only = "Only"
}
export declare enum TagQueryBehavior {
    /** Returns resources which have strictly all the tags */
    All = "All",
    /** Returns resources which have one or more of the tags */
    Any = "Any"
}
/** Passing empty Vec is the same as not filtering by that field */
export interface ResourceQuery<T> {
    names?: string[];
    templates?: TemplatesQueryBehavior;
    /** Pass Vec of tag ids or tag names */
    tags?: string[];
    /** 'All' or 'Any' */
    tag_behavior?: TagQueryBehavior;
    specific?: T;
}
export interface ActionQuerySpecifics {
}
export type ActionQuery = ResourceQuery<ActionQuerySpecifics>;
export type AlerterEndpoint = 
/** Send alert serialized to JSON to an http endpoint. */
{
    type: "Custom";
    params: CustomAlerterEndpoint;
}
/** Send alert to a Slack app */
 | {
    type: "Slack";
    params: SlackAlerterEndpoint;
}
/** Send alert to a Discord app */
 | {
    type: "Discord";
    params: DiscordAlerterEndpoint;
}
/** Send alert to Ntfy */
 | {
    type: "Ntfy";
    params: NtfyAlerterEndpoint;
}
/** Send alert to Pushover */
 | {
    type: "Pushover";
    params: PushoverAlerterEndpoint;
};
/** Used to reference a specific resource across all resource types */
export type ResourceTarget = {
    type: "System";
    id: string;
} | {
    type: "Swarm";
    id: string;
} | {
    type: "Server";
    id: string;
} | {
    type: "Stack";
    id: string;
} | {
    type: "Deployment";
    id: string;
} | {
    type: "Build";
    id: string;
} | {
    type: "Repo";
    id: string;
} | {
    type: "Procedure";
    id: string;
} | {
    type: "Action";
    id: string;
} | {
    type: "Builder";
    id: string;
} | {
    type: "Alerter";
    id: string;
} | {
    type: "ResourceSync";
    id: string;
};
/** Types of maintenance schedules */
export declare enum MaintenanceScheduleType {
    /** Daily at the specified time */
    Daily = "Daily",
    /** Weekly on the specified day and time */
    Weekly = "Weekly",
    /** One-time maintenance on a specific date and time */
    OneTime = "OneTime"
}
/** Represents a scheduled maintenance window */
export interface MaintenanceWindow {
    /** Name for the maintenance window (required) */
    name: string;
    /** Description of what maintenance is performed (optional) */
    description?: string;
    /**
     * The type of maintenance schedule:
     * - Daily (default)
     * - Weekly
     * - OneTime
     */
    schedule_type?: MaintenanceScheduleType;
    /** For Weekly schedules: Specify the day of the week (Monday, Tuesday, etc.) */
    day_of_week?: string;
    /** For OneTime window: ISO 8601 date format (YYYY-MM-DD) */
    date?: string;
    /** Start hour in 24-hour format (0-23) (optional, defaults to 0) */
    hour?: number;
    /** Start minute (0-59) (optional, defaults to 0) */
    minute?: number;
    /** Duration of the maintenance window in minutes (required) */
    duration_minutes: number;
    /**
     * Timezone for maintenance window specificiation.
     * If empty, will use Core timezone.
     */
    timezone?: string;
    /** Whether this maintenance window is currently enabled */
    enabled: boolean;
}
export interface AlerterConfig {
    /** Whether the alerter is enabled */
    enabled?: boolean;
    /**
     * Where to route the alert messages.
     *
     * Default: Custom endpoint `http://localhost:7000`
     */
    endpoint?: AlerterEndpoint;
    /**
     * Only send specific alert types.
     * If empty, will send all alert types.
     */
    alert_types?: AlertData["type"][];
    /**
     * Only send alerts on specific resources.
     * If empty, will send alerts for all resources.
     */
    resources?: ResourceTarget[];
    /** DON'T send alerts on these resources. */
    except_resources?: ResourceTarget[];
    /** Scheduled maintenance windows during which alerts will be suppressed. */
    maintenance_windows?: MaintenanceWindow[];
}
export type Alerter = Resource<AlerterConfig, undefined>;
export interface AlerterListItemInfo {
    /** Whether alerter is enabled for sending alerts */
    enabled: boolean;
    /** The type of the alerter, eg. `Slack`, `Custom` */
    endpoint_type: AlerterEndpoint["type"];
}
export type AlerterListItem = ResourceListItem<AlerterListItemInfo>;
export interface AlerterQuerySpecifics {
    /**
     * Filter alerters by enabled.
     * - `None`: Don't filter by enabled
     * - `Some(true)`: Only include alerts with `enabled: true`
     * - `Some(false)`: Only include alerts with `enabled: false`
     */
    enabled?: boolean;
    /**
     * Only include alerters with these endpoint types.
     * If empty, don't filter by enpoint type.
     */
    types: AlerterEndpoint["type"][];
}
export type AlerterQuery = ResourceQuery<AlerterQuerySpecifics>;
export type BatchExecutionResponseItem = {
    status: "Ok";
    data: Update;
} | {
    status: "Err";
    data: BatchExecutionResponseItemErr;
};
export type BatchExecutionResponse = BatchExecutionResponseItem[];
export declare enum Operation {
    None = "None",
    CreateSwarm = "CreateSwarm",
    UpdateSwarm = "UpdateSwarm",
    RenameSwarm = "RenameSwarm",
    DeleteSwarm = "DeleteSwarm",
    RemoveSwarmNodes = "RemoveSwarmNodes",
    RemoveSwarmStacks = "RemoveSwarmStacks",
    RemoveSwarmServices = "RemoveSwarmServices",
    RemoveSwarmConfigs = "RemoveSwarmConfigs",
    RemoveSwarmSecrets = "RemoveSwarmSecrets",
    CreateServer = "CreateServer",
    UpdateServer = "UpdateServer",
    UpdateServerKey = "UpdateServerKey",
    DeleteServer = "DeleteServer",
    RenameServer = "RenameServer",
    StartContainer = "StartContainer",
    RestartContainer = "RestartContainer",
    PauseContainer = "PauseContainer",
    UnpauseContainer = "UnpauseContainer",
    StopContainer = "StopContainer",
    DestroyContainer = "DestroyContainer",
    StartAllContainers = "StartAllContainers",
    RestartAllContainers = "RestartAllContainers",
    PauseAllContainers = "PauseAllContainers",
    UnpauseAllContainers = "UnpauseAllContainers",
    StopAllContainers = "StopAllContainers",
    PruneContainers = "PruneContainers",
    CreateNetwork = "CreateNetwork",
    DeleteNetwork = "DeleteNetwork",
    PruneNetworks = "PruneNetworks",
    DeleteImage = "DeleteImage",
    PruneImages = "PruneImages",
    DeleteVolume = "DeleteVolume",
    PruneVolumes = "PruneVolumes",
    PruneDockerBuilders = "PruneDockerBuilders",
    PruneBuildx = "PruneBuildx",
    PruneSystem = "PruneSystem",
    CreateStack = "CreateStack",
    UpdateStack = "UpdateStack",
    RenameStack = "RenameStack",
    DeleteStack = "DeleteStack",
    WriteStackContents = "WriteStackContents",
    RefreshStackCache = "RefreshStackCache",
    PullStack = "PullStack",
    DeployStack = "DeployStack",
    StartStack = "StartStack",
    RestartStack = "RestartStack",
    PauseStack = "PauseStack",
    UnpauseStack = "UnpauseStack",
    StopStack = "StopStack",
    DestroyStack = "DestroyStack",
    RunStackService = "RunStackService",
    DeployStackService = "DeployStackService",
    PullStackService = "PullStackService",
    StartStackService = "StartStackService",
    RestartStackService = "RestartStackService",
    PauseStackService = "PauseStackService",
    UnpauseStackService = "UnpauseStackService",
    StopStackService = "StopStackService",
    DestroyStackService = "DestroyStackService",
    CreateDeployment = "CreateDeployment",
    UpdateDeployment = "UpdateDeployment",
    RenameDeployment = "RenameDeployment",
    DeleteDeployment = "DeleteDeployment",
    Deploy = "Deploy",
    PullDeployment = "PullDeployment",
    StartDeployment = "StartDeployment",
    RestartDeployment = "RestartDeployment",
    PauseDeployment = "PauseDeployment",
    UnpauseDeployment = "UnpauseDeployment",
    StopDeployment = "StopDeployment",
    DestroyDeployment = "DestroyDeployment",
    CreateBuild = "CreateBuild",
    UpdateBuild = "UpdateBuild",
    RenameBuild = "RenameBuild",
    DeleteBuild = "DeleteBuild",
    RunBuild = "RunBuild",
    CancelBuild = "CancelBuild",
    WriteDockerfile = "WriteDockerfile",
    CreateRepo = "CreateRepo",
    UpdateRepo = "UpdateRepo",
    RenameRepo = "RenameRepo",
    DeleteRepo = "DeleteRepo",
    CloneRepo = "CloneRepo",
    PullRepo = "PullRepo",
    BuildRepo = "BuildRepo",
    CancelRepoBuild = "CancelRepoBuild",
    CreateProcedure = "CreateProcedure",
    UpdateProcedure = "UpdateProcedure",
    RenameProcedure = "RenameProcedure",
    DeleteProcedure = "DeleteProcedure",
    RunProcedure = "RunProcedure",
    CreateAction = "CreateAction",
    UpdateAction = "UpdateAction",
    RenameAction = "RenameAction",
    DeleteAction = "DeleteAction",
    RunAction = "RunAction",
    CreateResourceSync = "CreateResourceSync",
    UpdateResourceSync = "UpdateResourceSync",
    RenameResourceSync = "RenameResourceSync",
    DeleteResourceSync = "DeleteResourceSync",
    WriteSyncContents = "WriteSyncContents",
    CommitSync = "CommitSync",
    RunSync = "RunSync",
    CreateBuilder = "CreateBuilder",
    UpdateBuilder = "UpdateBuilder",
    RenameBuilder = "RenameBuilder",
    DeleteBuilder = "DeleteBuilder",
    CreateAlerter = "CreateAlerter",
    UpdateAlerter = "UpdateAlerter",
    RenameAlerter = "RenameAlerter",
    DeleteAlerter = "DeleteAlerter",
    TestAlerter = "TestAlerter",
    SendAlert = "SendAlert",
    ClearRepoCache = "ClearRepoCache",
    BackupCoreDatabase = "BackupCoreDatabase",
    GlobalAutoUpdate = "GlobalAutoUpdate",
    RotateAllServerKeys = "RotateAllServerKeys",
    RotateCoreKeys = "RotateCoreKeys",
    CreateVariable = "CreateVariable",
    UpdateVariableValue = "UpdateVariableValue",
    DeleteVariable = "DeleteVariable",
    CreateGitProviderAccount = "CreateGitProviderAccount",
    UpdateGitProviderAccount = "UpdateGitProviderAccount",
    DeleteGitProviderAccount = "DeleteGitProviderAccount",
    CreateDockerRegistryAccount = "CreateDockerRegistryAccount",
    UpdateDockerRegistryAccount = "UpdateDockerRegistryAccount",
    DeleteDockerRegistryAccount = "DeleteDockerRegistryAccount"
}
/** Represents the output of some command being run */
export interface Log {
    /** A label for the log */
    stage: string;
    /** The command which was executed */
    command: string;
    /** The output of the command in the standard channel */
    stdout: string;
    /** The output of the command in the error channel */
    stderr: string;
    /** Whether the command run was successful */
    success: boolean;
    /** The start time of the command execution */
    start_ts: I64;
    /** The end time of the command execution */
    end_ts: I64;
}
/** An update's status */
export declare enum UpdateStatus {
    /** The run is in the system but hasn't started yet */
    Queued = "Queued",
    /** The run is currently running */
    InProgress = "InProgress",
    /** The run is complete */
    Complete = "Complete"
}
export interface Version {
    major: number;
    minor: number;
    patch: number;
}
/** Represents an action performed by Komodo. */
export interface Update {
    /**
     * The Mongo ID of the update.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized Update) }`
     */
    _id?: MongoId;
    /** The operation performed */
    operation: Operation;
    /** The time the operation started */
    start_ts: I64;
    /** Whether the operation was successful */
    success: boolean;
    /**
     * The user id that triggered the update.
     *
     * Also can take these values for operations triggered automatically:
     * - `Procedure`: The operation was triggered as part of a procedure run
     * - `Github`: The operation was triggered by a github webhook
     * - `Auto Redeploy`: The operation (always `Deploy`) was triggered by an attached build finishing.
     */
    operator: string;
    /** The target resource to which this update refers */
    target: ResourceTarget;
    /** Logs produced as the operation is performed */
    logs: Log[];
    /** The time the operation completed. */
    end_ts?: I64;
    /**
     * The status of the update
     * - `Queued`
     * - `InProgress`
     * - `Complete`
     */
    status: UpdateStatus;
    /** An optional version on the update, ie build version or deployed version. */
    version?: Version;
    /** An optional commit hash associated with the update, ie cloned hash or deployed hash. */
    commit_hash?: string;
    /** Some unstructured, operation specific data. Not for general usage. */
    other_data?: string;
    /** If the update is for resource config update, give the previous toml contents */
    prev_toml?: string;
    /** If the update is for resource config update, give the current (at time of Update) toml contents */
    current_toml?: string;
}
export type BoxUpdate = Update;
/** Configuration for an image registry */
export interface ImageRegistryConfig {
    /**
     * Specify the registry provider domain, eg `docker.io`.
     * If not provided, will not push to any registry.
     */
    domain?: string;
    /** Specify an account to use with the registry. */
    account?: string;
    /**
     * Optional. Specify an organization to push the image under.
     * Empty string means no organization.
     */
    organization?: string;
}
export interface SystemCommand {
    path?: string;
    command?: string;
}
/** The build configuration. */
export interface BuildConfig {
    /** Which builder is used to build the image. */
    builder_id?: string;
    /** The current version of the build. */
    version?: Version;
    /**
     * Whether to automatically increment the patch on every build.
     * Default is `true`
     */
    auto_increment_version: boolean;
    /**
     * An alternate name for the image pushed to the repository.
     * If this is empty, it will use the build name.
     *
     * Can be used in conjunction with `image_tag` to direct multiple builds
     * with different configs to push to the same image registry, under different,
     * independantly versioned tags.
     */
    image_name?: string;
    /**
     * An extra tag put after the build version, for the image pushed to the repository.
     * Eg. in image tag of `aarch64` would push to moghtech/komodo-core:1.13.2-aarch64.
     * If this is empty, the image tag will just be the build version.
     *
     * Can be used in conjunction with `image_name` to direct multiple builds
     * with different configs to push to the same image registry, under different,
     * independantly versioned tags.
     */
    image_tag?: string;
    /** Push `:latest` / `:latest-image_tag` tags. */
    include_latest_tag: boolean;
    /** Push build version semver `:1.19.5` + `1.19` / `:1.19.5-image_tag` tags. */
    include_version_tags: boolean;
    /** Push commit hash `:a6v8h83` / `:a6v8h83-image_tag` tags. */
    include_commit_tag: boolean;
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
    /** Choose a Komodo Repo (Resource) to source the build files. */
    linked_repo?: string;
    /** The git provider domain. Default: github.com */
    git_provider: string;
    /**
     * Whether to use https to clone the repo (versus http). Default: true
     *
     * Note. Komodo does not currently support cloning repos via ssh.
     */
    git_https: boolean;
    /**
     * The git account used to access private repos.
     * Passing empty string can only clone public repos.
     *
     * Note. A token for the account must be available in the core config or the builder server's periphery config
     * for the configured git provider.
     */
    git_account?: string;
    /** The repo used as the source of the build. */
    repo?: string;
    /** The branch of the repo. */
    branch: string;
    /** Optionally set a specific commit hash. */
    commit?: string;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this build.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
    /**
     * If this is checked, the build will source the files on the host.
     * Use `build_path` and `dockerfile_path` to specify the path on the host.
     * This is useful for those who wish to setup their files on the host,
     * rather than defining the contents in UI or in a git repo.
     */
    files_on_host?: boolean;
    /**
     * The path of the docker build context relative to the root of the repo.
     * Default: "." (the root of the repo).
     */
    build_path: string;
    /** The path of the dockerfile relative to the build path. */
    dockerfile_path: string;
    /**
     * Configuration for the registry/s to push the built image to.
     * The first registry in this list will be used with attached Deployments.
     */
    image_registry?: ImageRegistryConfig[];
    /** Whether to skip secret interpolation in the build_args. */
    skip_secret_interp?: boolean;
    /** Whether to use buildx to build (eg `docker buildx build ...`) */
    use_buildx?: boolean;
    /** Any extra docker cli arguments to be included in the build command */
    extra_args?: string[];
    /** The optional command run after repo clone and before docker build. */
    pre_build?: SystemCommand;
    /**
     * UI defined dockerfile contents.
     * Supports variable / secret interpolation.
     */
    dockerfile?: string;
    /**
     * Docker build arguments.
     *
     * These values are visible in the final image by running `docker inspect`.
     */
    build_args?: string;
    /**
     * Secret arguments.
     *
     * These values remain hidden in the final image by using
     * docker secret mounts. See <https://docs.docker.com/build/building/secrets>.
     *
     * The values can be used in RUN commands:
     * ```sh
     * RUN --mount=type=secret,id=SECRET_KEY \
     * SECRET_KEY=$(cat /run/secrets/SECRET_KEY) ...
     * ```
     */
    secret_args?: string;
    /** Docker labels */
    labels?: string;
}
export interface BuildInfo {
    /** The timestamp build was last built. */
    last_built_at: I64;
    /** Latest built short commit hash, or null. */
    built_hash?: string;
    /** Latest built commit message, or null. Only for repo based stacks */
    built_message?: string;
    /**
     * The last built dockerfile contents.
     * This is updated whenever Komodo successfully runs the build.
     */
    built_contents?: string;
    /** The absolute path to the file */
    remote_path?: string;
    /**
     * The remote dockerfile contents, whether on host or in repo.
     * This is updated whenever Komodo refreshes the build cache.
     * It will be empty if the dockerfile is defined directly in the build config.
     */
    remote_contents?: string;
    /** If there was an error in getting the remote contents, it will be here. */
    remote_error?: string;
    /** Latest remote short commit hash, or null. */
    latest_hash?: string;
    /** Latest remote commit message, or null */
    latest_message?: string;
}
export type Build = Resource<BuildConfig, BuildInfo>;
export declare enum BuildState {
    /** Currently building */
    Building = "Building",
    /** Last build successful (or never built) */
    Ok = "Ok",
    /** Last build failed */
    Failed = "Failed",
    /** Other case */
    Unknown = "Unknown"
}
export interface BuildListItemInfo {
    /** State of the build. Reflects whether most recent build successful. */
    state: BuildState;
    /** Unix timestamp in milliseconds of last build */
    last_built_at: I64;
    /** The current version of the build */
    version: Version;
    /** The builder attached to build. */
    builder_id: string;
    /** Whether build is in files on host mode. */
    files_on_host: boolean;
    /** Whether build has UI defined dockerfile contents */
    dockerfile_contents: boolean;
    /** Linked repo, if one is attached. */
    linked_repo: string;
    /** The git provider domain */
    git_provider: string;
    /** The repo used as the source of the build */
    repo: string;
    /** The branch of the repo */
    branch: string;
    /** Full link to the repo. */
    repo_link: string;
    /** Latest built short commit hash, or null. */
    built_hash?: string;
    /** Latest short commit hash, or null. Only for repo based stacks */
    latest_hash?: string;
    /** The first listed image registry domain */
    image_registry_domain?: string;
}
export type BuildListItem = ResourceListItem<BuildListItemInfo>;
export interface BuildQuerySpecifics {
    builder_ids?: string[];
    repos?: string[];
    /**
     * query for builds last built more recently than this timestamp
     * defaults to 0 which is a no op
     */
    built_since?: I64;
}
export type BuildQuery = ResourceQuery<BuildQuerySpecifics>;
export type BuilderConfig = 
/** Use a Periphery address as a Builder. */
{
    type: "Url";
    params: UrlBuilderConfig;
}
/** Use a connected server as a Builder. */
 | {
    type: "Server";
    params: ServerBuilderConfig;
}
/** Use EC2 instances spawned on demand as a Builder. */
 | {
    type: "Aws";
    params: AwsBuilderConfig;
};
export type Builder = Resource<BuilderConfig, undefined>;
export interface BuilderListItemInfo {
    /** 'Url', 'Server', or 'Aws' */
    builder_type: string;
    /**
     * If 'Url': null
     * If 'Server': the server id
     * If 'Aws': the instance type (eg. c5.xlarge)
     */
    instance_type?: string;
}
export type BuilderListItem = ResourceListItem<BuilderListItemInfo>;
export interface BuilderQuerySpecifics {
}
export type BuilderQuery = ResourceQuery<BuilderQuerySpecifics>;
/** A wrapper for all Komodo exections. */
export type Execution = 
/** The "null" execution. Does nothing. */
{
    type: "None";
    params: NoData;
}
/** Deploy the target stack. (alias: `stack`, `st`) */
 | {
    type: "DeployStack";
    params: DeployStack;
} | {
    type: "BatchDeployStack";
    params: BatchDeployStack;
} | {
    type: "DeployStackIfChanged";
    params: DeployStackIfChanged;
} | {
    type: "BatchDeployStackIfChanged";
    params: BatchDeployStackIfChanged;
} | {
    type: "PullStack";
    params: PullStack;
} | {
    type: "BatchPullStack";
    params: BatchPullStack;
} | {
    type: "StartStack";
    params: StartStack;
} | {
    type: "RestartStack";
    params: RestartStack;
} | {
    type: "PauseStack";
    params: PauseStack;
} | {
    type: "UnpauseStack";
    params: UnpauseStack;
} | {
    type: "StopStack";
    params: StopStack;
} | {
    type: "DestroyStack";
    params: DestroyStack;
} | {
    type: "BatchDestroyStack";
    params: BatchDestroyStack;
} | {
    type: "RunStackService";
    params: RunStackService;
}
/** Deploy the target deployment. (alias: `dp`) */
 | {
    type: "Deploy";
    params: Deploy;
} | {
    type: "BatchDeploy";
    params: BatchDeploy;
} | {
    type: "PullDeployment";
    params: PullDeployment;
} | {
    type: "StartDeployment";
    params: StartDeployment;
} | {
    type: "RestartDeployment";
    params: RestartDeployment;
} | {
    type: "PauseDeployment";
    params: PauseDeployment;
} | {
    type: "UnpauseDeployment";
    params: UnpauseDeployment;
} | {
    type: "StopDeployment";
    params: StopDeployment;
} | {
    type: "DestroyDeployment";
    params: DestroyDeployment;
} | {
    type: "BatchDestroyDeployment";
    params: BatchDestroyDeployment;
}
/** Run the target build. (alias: `build`, `bd`) */
 | {
    type: "RunBuild";
    params: RunBuild;
} | {
    type: "BatchRunBuild";
    params: BatchRunBuild;
} | {
    type: "CancelBuild";
    params: CancelBuild;
}
/** Clone the target repo */
 | {
    type: "CloneRepo";
    params: CloneRepo;
} | {
    type: "BatchCloneRepo";
    params: BatchCloneRepo;
} | {
    type: "PullRepo";
    params: PullRepo;
} | {
    type: "BatchPullRepo";
    params: BatchPullRepo;
} | {
    type: "BuildRepo";
    params: BuildRepo;
} | {
    type: "BatchBuildRepo";
    params: BatchBuildRepo;
} | {
    type: "CancelRepoBuild";
    params: CancelRepoBuild;
}
/** Run the target procedure. (alias: `procedure`, `pr`) */
 | {
    type: "RunProcedure";
    params: RunProcedure;
} | {
    type: "BatchRunProcedure";
    params: BatchRunProcedure;
}
/** Run the target action. (alias: `action`, `ac`) */
 | {
    type: "RunAction";
    params: RunAction;
} | {
    type: "BatchRunAction";
    params: BatchRunAction;
}
/** Execute a Resource Sync. (alias: `sync`) */
 | {
    type: "RunSync";
    params: RunSync;
}
/** Commit a Resource Sync. (alias: `commit`) */
 | {
    type: "CommitSync";
    params: CommitSync;
} | {
    type: "TestAlerter";
    params: TestAlerter;
} | {
    type: "SendAlert";
    params: SendAlert;
} | {
    type: "StartContainer";
    params: StartContainer;
} | {
    type: "RestartContainer";
    params: RestartContainer;
} | {
    type: "PauseContainer";
    params: PauseContainer;
} | {
    type: "UnpauseContainer";
    params: UnpauseContainer;
} | {
    type: "StopContainer";
    params: StopContainer;
} | {
    type: "DestroyContainer";
    params: DestroyContainer;
} | {
    type: "StartAllContainers";
    params: StartAllContainers;
} | {
    type: "RestartAllContainers";
    params: RestartAllContainers;
} | {
    type: "PauseAllContainers";
    params: PauseAllContainers;
} | {
    type: "UnpauseAllContainers";
    params: UnpauseAllContainers;
} | {
    type: "StopAllContainers";
    params: StopAllContainers;
} | {
    type: "PruneContainers";
    params: PruneContainers;
} | {
    type: "DeleteNetwork";
    params: DeleteNetwork;
} | {
    type: "PruneNetworks";
    params: PruneNetworks;
} | {
    type: "DeleteImage";
    params: DeleteImage;
} | {
    type: "PruneImages";
    params: PruneImages;
} | {
    type: "DeleteVolume";
    params: DeleteVolume;
} | {
    type: "PruneVolumes";
    params: PruneVolumes;
} | {
    type: "PruneDockerBuilders";
    params: PruneDockerBuilders;
} | {
    type: "PruneBuildx";
    params: PruneBuildx;
} | {
    type: "PruneSystem";
    params: PruneSystem;
} | {
    type: "RemoveSwarmNodes";
    params: RemoveSwarmNodes;
} | {
    type: "RemoveSwarmStacks";
    params: RemoveSwarmStacks;
} | {
    type: "RemoveSwarmServices";
    params: RemoveSwarmServices;
} | {
    type: "RemoveSwarmConfigs";
    params: RemoveSwarmConfigs;
} | {
    type: "RemoveSwarmSecrets";
    params: RemoveSwarmSecrets;
} | {
    type: "ClearRepoCache";
    params: ClearRepoCache;
} | {
    type: "BackupCoreDatabase";
    params: BackupCoreDatabase;
} | {
    type: "GlobalAutoUpdate";
    params: GlobalAutoUpdate;
} | {
    type: "RotateAllServerKeys";
    params: RotateAllServerKeys;
} | {
    type: "RotateCoreKeys";
    params: RotateCoreKeys;
} | {
    type: "Sleep";
    params: Sleep;
};
/** Allows to enable / disabled procedures in the sequence / parallel vec on the fly */
export interface EnabledExecution {
    /** The execution request to run. */
    execution: Execution;
    /** Whether the execution is enabled to run in the procedure. */
    enabled: boolean;
}
/** A single stage of a procedure. Runs a list of executions in parallel. */
export interface ProcedureStage {
    /** A name for the procedure */
    name: string;
    /** Whether the stage should be run as part of the procedure. */
    enabled: boolean;
    /** The executions in the stage */
    executions?: EnabledExecution[];
}
/** Config for the [Procedure] */
export interface ProcedureConfig {
    /** The stages to be run by the procedure. */
    stages?: ProcedureStage[];
    /** Choose whether to specify schedule as regular CRON, or using the english to CRON parser. */
    schedule_format?: ScheduleFormat;
    /**
     * Optionally provide a schedule for the procedure to run on.
     *
     * There are 2 ways to specify a schedule:
     *
     * 1. Regular CRON expression:
     *
     * (second, minute, hour, day, month, day-of-week)
     * ```text
     * 0 0 0 1,15 * ?
     * ```
     *
     * 2. "English" expression via [english-to-cron](https://crates.io/crates/english-to-cron):
     *
     * ```text
     * at midnight on the 1st and 15th of the month
     * ```
     */
    schedule?: string;
    /**
     * Whether schedule is enabled if one is provided.
     * Can be used to temporarily disable the schedule.
     */
    schedule_enabled: boolean;
    /**
     * Optional. A TZ Identifier. If not provided, will use Core local timezone.
     * https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
     */
    schedule_timezone?: string;
    /** Whether to send alerts when the schedule was run. */
    schedule_alert: boolean;
    /** Whether to send alerts when this procedure fails. */
    failure_alert: boolean;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this procedure.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
}
/**
 * Procedures run a series of stages sequentially, where
 * each stage runs executions in parallel.
 */
export type Procedure = Resource<ProcedureConfig, undefined>;
export type CopyProcedureResponse = Procedure;
/** Response for [CreateApiKey]. */
export interface CreateApiKeyResponse {
    /** X-API-KEY */
    key: string;
    /**
     * X-API-SECRET
     *
     * Note.
     * There is no way to get the secret again after it is distributed in this message
     */
    secret: string;
}
export type CreateApiKeyForServiceUserResponse = CreateApiKeyResponse;
/** Configuration to access private image repositories on various registries. */
export interface DockerRegistryAccount {
    /**
     * The Mongo ID of the docker registry account.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of DockerRegistryAccount) }`
     */
    _id?: MongoId;
    /**
     * The domain of the provider.
     *
     * For docker registry, this can include 'http://...',
     * however this is not recommended and won't work unless "insecure registries" are enabled
     * on your hosts. See <https://docs.docker.com/reference/cli/dockerd/#insecure-registries>.
     */
    domain: string;
    /** The account username */
    username?: string;
    /**
     * The token in plain text on the db.
     * If the database / host can be accessed this is insecure.
     */
    token?: string;
}
export type CreateDockerRegistryAccountResponse = DockerRegistryAccount;
/**
 * Configuration to access private git repos from various git providers.
 * Note. Cannot create two accounts with the same domain and username.
 */
export interface GitProviderAccount {
    /**
     * The Mongo ID of the git provider account.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized User) }`
     */
    _id?: MongoId;
    /**
     * The domain of the provider.
     *
     * For git, this cannot include the protocol eg 'http://',
     * which is controlled with 'https' field.
     */
    domain: string;
    /** Whether git provider is accessed over http or https. */
    https: boolean;
    /** The account username */
    username?: string;
    /**
     * The token in plain text on the db.
     * If the database / host can be accessed this is insecure.
     */
    token?: string;
}
export type CreateGitProviderAccountResponse = GitProviderAccount;
export type UserConfig = 
/** User that logs in with username / password */
{
    type: "Local";
    data: {
        password: string;
    };
}
/** User that logs in via Google Oauth */
 | {
    type: "Google";
    data: {
        google_id: string;
        avatar: string;
    };
}
/** User that logs in via Github Oauth */
 | {
    type: "Github";
    data: {
        github_id: string;
        avatar: string;
    };
}
/** User that logs in via Oidc provider */
 | {
    type: "Oidc";
    data: {
        provider: string;
        user_id: string;
    };
}
/** Non-human managed user, can have it's own permissions / api keys */
 | {
    type: "Service";
    data: {
        description: string;
    };
};
export interface User {
    /**
     * The Mongo ID of the User.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of User schema) }`
     */
    _id?: MongoId;
    /** The globally unique username for the user. */
    username: string;
    /** Whether user is enabled / able to access the api. */
    enabled?: boolean;
    /** Can give / take other users admin priviledges. */
    super_admin?: boolean;
    /** Whether the user has global admin permissions. */
    admin?: boolean;
    /** Whether the user has permission to create servers. */
    create_server_permissions?: boolean;
    /** Whether the user has permission to create builds */
    create_build_permissions?: boolean;
    /** The user-type specific config. */
    config: UserConfig;
    /** When the user last opened updates dropdown. */
    last_update_view?: I64;
    /** Recently viewed ids */
    recents?: Record<ResourceTarget["type"], string[]>;
    /** Give the user elevated permissions on all resources of a certain type */
    all?: Record<ResourceTarget["type"], PermissionLevelAndSpecifics | PermissionLevel>;
    updated_at?: I64;
}
export type CreateLocalUserResponse = User;
export type CreateProcedureResponse = Procedure;
export type CreateServiceUserResponse = User;
/**
 * A non-secret global variable which can be interpolated into deployment
 * environment variable values and build argument values.
 */
export interface Variable {
    /**
     * Unique name associated with the variable.
     * Instances of '[[variable.name]]' in value will be replaced with 'variable.value'.
     */
    name: string;
    /** A description for the variable. */
    description?: string;
    /** The value associated with the variable. */
    value?: string;
    /**
     * If marked as secret, the variable value will be hidden in updates / logs.
     * Additionally the value will not be served in read requests by non admin users.
     *
     * Note that the value is NOT encrypted in the database, and will likely show up in database logs.
     * The security of these variables comes down to the security
     * of the database (system level encryption, network isolation, etc.)
     */
    is_secret?: boolean;
}
export type CreateVariableResponse = Variable;
export type DeleteApiKeyForServiceUserResponse = NoData;
export type DeleteApiKeyResponse = NoData;
export type DeleteDockerRegistryAccountResponse = DockerRegistryAccount;
export type DeleteGitProviderAccountResponse = GitProviderAccount;
/**
 * An public key used to authenticate new Periphery -> Core connections
 * to join Komodo as a newly created Server.
 *
 * Server onboarding keys correspond to private / public key pairs.
 * While the public key is stored, the private key will only be returned to the user,
 * The private key will not be stored or available afterwards, just like the api key "secret".
 */
export interface OnboardingKey {
    /** Unique public key associated the creation private key. */
    public_key: string;
    /** Disable the onboarding key when not in use. */
    enabled?: boolean;
    /** Expiry of key, or 0 if never expires */
    expires?: I64;
    /** Name associated with the api key for management */
    name?: string;
    /** The [Server](crate::entities::server::Server) ids onboarded by this Creation Key */
    onboarded?: string[];
    /** Timestamp of key creation */
    created_at?: I64;
    /** Default tags to give to Servers created with this key. */
    tags?: string[];
    /**
     * Optional. If specified, copy this Server config when initializing
     * the Server.
     */
    copy_server?: string;
    /** Also create a Builder for the Server. */
    create_builder?: boolean;
}
export type DeleteOnboardingKeyResponse = OnboardingKey;
export type DeleteProcedureResponse = Procedure;
export type DeleteUserResponse = User;
export type DeleteVariableResponse = Variable;
export type DeploymentImage = 
/** Deploy any external image. */
{
    type: "Image";
    params: {
        /** The docker image, can be from any registry that works with docker and that the host server can reach. */
        image?: string;
    };
}
/** Deploy a Komodo Build. */
 | {
    type: "Build";
    params: {
        /** The id of the Build */
        build_id?: string;
        /**
         * Use a custom / older version of the image produced by the build.
         * if version is 0.0.0, this means `latest` image.
         */
        version?: Version;
    };
};
export declare enum RestartMode {
    NoRestart = "no",
    OnFailure = "on-failure",
    Always = "always",
    UnlessStopped = "unless-stopped"
}
export declare enum TerminationSignal {
    SigHup = "SIGHUP",
    SigInt = "SIGINT",
    SigQuit = "SIGQUIT",
    SigTerm = "SIGTERM"
}
export interface DeploymentConfig {
    /**
     * The Swarm to deploy the Deployment on (as a Swarm Service), setting the Deployment into Swarm mode.
     *
     * Note. If both swarm_id and server_id are set,
     * swarm_id overrides server_id and the Deployment will be in Swarm mode.
     */
    swarm_id?: string;
    /**
     * The Server to deploy the Deployment on, setting the Deployment into Container mode.
     *
     * Note. If both swarm_id and server_id are set,
     * swarm_id overrides server_id and the Deployment will be in Swarm mode.
     */
    server_id?: string;
    /**
     * The image which the deployment deploys.
     * Can either be a user inputted image, or a Komodo Build.
     */
    image?: DeploymentImage;
    /**
     * Configure the account used to pull the image from the registry.
     * Used with `docker login`.
     *
     * - If the field is empty string, will use the same account config as the build, or none at all if using image.
     * - If the field contains an account, a token for the account must be available.
     * - Will get the registry domain from the build / image
     */
    image_registry_account?: string;
    /** Whether to skip secret interpolation into the deployment environment variables. */
    skip_secret_interp?: boolean;
    /** Whether to redeploy the deployment whenever the attached build finishes. */
    redeploy_on_build?: boolean;
    /** Whether to poll for any updates to the image. */
    poll_for_updates?: boolean;
    /**
     * Whether to automatically redeploy when
     * newer a image is found. Will implicitly
     * enable `poll_for_updates`, you don't need to
     * enable both.
     */
    auto_update?: boolean;
    /** Whether to send ContainerStateChange alerts for this deployment. */
    send_alerts: boolean;
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
    /**
     * The network attached to the container.
     * Default is `host`.
     */
    network: string;
    /** The restart mode given to the container. */
    restart?: RestartMode;
    /**
     * This is interpolated at the end of the `docker run` command,
     * which means they are either passed to the containers inner process,
     * or replaces the container command, depending on use of ENTRYPOINT or CMD in dockerfile.
     * Empty is no command.
     */
    command?: string;
    /**
     * The number of replicas for the Service.
     *
     * Note. Only used in Swarm mode.
     */
    replicas: number;
    /** The default termination signal to use to stop the deployment. Defaults to SigTerm (default docker signal). */
    termination_signal?: TerminationSignal;
    /** The termination timeout. */
    termination_timeout: number;
    /**
     * Extra args which are interpolated into the
     * `docker run` / `docker service create` command,
     * and affect the container configuration.
     *
     * - Container ref: https://docs.docker.com/reference/cli/docker/container/run/#options
     * - Swarm Service ref: https://docs.docker.com/reference/cli/docker/service/create/#options
     */
    extra_args?: string[];
    /**
     * Labels attached to various termination signal options.
     * Used to specify different shutdown functionality depending
     * on the termination signal.
     */
    term_signal_labels?: string;
    /**
     * The container port mapping.
     * Irrelevant if container network is `host`.
     * Maps ports on host to ports on container.
     */
    ports?: string;
    /**
     * The container volume mapping.
     * Maps files / folders on host to files / folders in container.
     */
    volumes?: string;
    /** The environment variables passed to the container / service. */
    environment?: string;
    /** The docker labels given to the container. */
    labels?: string;
}
export type Deployment = Resource<DeploymentConfig, undefined>;
/**
 * Variants de/serialized from/to snake_case.
 *
 * Eg.
 * - NotDeployed -> not_deployed
 * - Restarting -> restarting
 * - Running -> running.
 */
export declare enum DeploymentState {
    /** The deployment is currently re/deploying */
    Deploying = "deploying",
    /** Container / Service is running */
    Running = "running",
    /** Server mode only. Container is created but not running. */
    Created = "created",
    /** Server mode only. Container is in restart loop */
    Restarting = "restarting",
    /** Server mode only. Container is being removed */
    Removing = "removing",
    /** Server mode only. Container is paused */
    Paused = "paused",
    /** Server mode only. Container is exited */
    Exited = "exited",
    /** Server mode only. Container is dead */
    Dead = "dead",
    /** Swarm mode only. Some tasks don't match their desired state. */
    Unhealthy = "unhealthy",
    /** The deployment is not deployed (no matching Container / Service) */
    NotDeployed = "not_deployed",
    /** Server / Swarm not reachable for status */
    Unknown = "unknown"
}
export interface DeploymentListItemInfo {
    /** The state of the deployment / underlying docker container. */
    state: DeploymentState;
    /** The status of the docker container (eg. up 12 hours, exited 5 minutes ago.) */
    status?: string;
    /** The image attached to the deployment. */
    image: string;
    /** Whether there is a newer image available at the same tag. */
    update_available: boolean;
    /** The server that deployment sits on. */
    server_id: string;
    /** An attached Komodo Build, if it exists. */
    build_id?: string;
}
export type DeploymentListItem = ResourceListItem<DeploymentListItemInfo>;
export interface DeploymentQuerySpecifics {
    /**
     * Query only for Deployments on these Servers.
     * If empty, does not filter by Server.
     * Only accepts Server id (not name).
     */
    server_ids?: string[];
    /**
     * Query only for Deployments with these Builds attached.
     * If empty, does not filter by Build.
     * Only accepts Build id (not name).
     */
    build_ids?: string[];
    /** Query only for Deployments with available image updates. */
    update_available?: boolean;
}
export type DeploymentQuery = ResourceQuery<DeploymentQuerySpecifics>;
/** JSON containing an authentication token. */
export interface JwtResponse {
    /** User ID for signed in user. */
    user_id: string;
    /** A token the user can use to authenticate their requests. */
    jwt: string;
}
/** Response for [ExchangeForJwt]. */
export type ExchangeForJwtResponse = JwtResponse;
/** Response containing pretty formatted toml contents. */
export interface TomlResponse {
    toml: string;
}
export type ExportAllResourcesToTomlResponse = TomlResponse;
export type ExportResourcesToTomlResponse = TomlResponse;
export type FindUserResponse = User;
export interface GenericResourcesInnerNamedResourceSpec {
    Kind?: string;
    Value?: string;
}
export interface GenericResourcesInnerDiscreteResourceSpec {
    Kind?: string;
    Value?: I64;
}
export interface GenericResourcesInner {
    NamedResourceSpec?: GenericResourcesInnerNamedResourceSpec;
    DiscreteResourceSpec?: GenericResourcesInnerDiscreteResourceSpec;
}
/** User-defined resources can be either Integer resources (e.g, `SSD=3`) or String resources (e.g, `GPU=UUID1`). */
export type GenericResources = GenericResourcesInner[];
export interface ActionActionState {
    /** Number of instances of the Action currently running */
    running: number;
}
export type GetActionActionStateResponse = ActionActionState;
export type GetActionResponse = Action;
/** Severity level of problem. */
export declare enum SeverityLevel {
    /**
     * No problem.
     *
     * Aliases: ok, low, l
     */
    Ok = "OK",
    /**
     * Problem is imminent.
     *
     * Aliases: warning, w, medium, m
     */
    Warning = "WARNING",
    /**
     * Problem fully realized.
     *
     * Aliases: critical, c, high, h
     */
    Critical = "CRITICAL"
}
/** The variants of data related to the alert. */
export type AlertData = 
/** A null alert */
{
    type: "None";
    data: {};
}
/**
 * The user triggered a test of the
 * Alerter configuration.
 */
 | {
    type: "Test";
    data: {
        /** The id of the alerter */
        id: string;
        /** The name of the alerter */
        name: string;
    };
}
/** A server could not be reached. */
 | {
    type: "ServerUnreachable";
    data: {
        /** The id of the server */
        id: string;
        /** The name of the server */
        name: string;
        /** The region of the server */
        region?: string;
        /** The error data */
        err?: _Serror;
    };
}
/** A server has high CPU usage. */
 | {
    type: "ServerCpu";
    data: {
        /** The id of the server */
        id: string;
        /** The name of the server */
        name: string;
        /** The region of the server */
        region?: string;
        /** The cpu usage percentage */
        percentage: number;
    };
}
/** A server has high memory usage. */
 | {
    type: "ServerMem";
    data: {
        /** The id of the server */
        id: string;
        /** The name of the server */
        name: string;
        /** The region of the server */
        region?: string;
        /** The used memory */
        used_gb: number;
        /** The total memory */
        total_gb: number;
    };
}
/** A server has high disk usage. */
 | {
    type: "ServerDisk";
    data: {
        /** The id of the server */
        id: string;
        /** The name of the server */
        name: string;
        /** The region of the server */
        region?: string;
        /** The mount path of the disk */
        path: string;
        /** The used portion of the disk in GB */
        used_gb: number;
        /** The total size of the disk in GB */
        total_gb: number;
    };
}
/** A server has a version mismatch with the core. */
 | {
    type: "ServerVersionMismatch";
    data: {
        /** The id of the server */
        id: string;
        /** The name of the server */
        name: string;
        /** The region of the server */
        region?: string;
        /** The actual server version */
        server_version: string;
        /** The core version */
        core_version: string;
    };
}
/** A container's state has changed unexpectedly. */
 | {
    type: "ContainerStateChange";
    data: {
        /** The id of the deployment */
        id: string;
        /** The name of the deployment */
        name: string;
        /** The server id of server that the deployment is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** The previous container state */
        from: DeploymentState;
        /** The current container state */
        to: DeploymentState;
    };
}
/** A Deployment has an image update available */
 | {
    type: "DeploymentImageUpdateAvailable";
    data: {
        /** The id of the deployment */
        id: string;
        /** The name of the deployment */
        name: string;
        /** The server id of server that the deployment is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** The image with update */
        image: string;
    };
}
/** A Deployment has an image update available */
 | {
    type: "DeploymentAutoUpdated";
    data: {
        /** The id of the deployment */
        id: string;
        /** The name of the deployment */
        name: string;
        /** The server id of server that the deployment is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** The updated image */
        image: string;
    };
}
/** A stack's state has changed unexpectedly. */
 | {
    type: "StackStateChange";
    data: {
        /** The id of the stack */
        id: string;
        /** The name of the stack */
        name: string;
        /** The server id of server that the stack is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** The previous stack state */
        from: StackState;
        /** The current stack state */
        to: StackState;
    };
}
/** A Stack has an image update available */
 | {
    type: "StackImageUpdateAvailable";
    data: {
        /** The id of the stack */
        id: string;
        /** The name of the stack */
        name: string;
        /** The server id of server that the stack is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** The service name to update */
        service: string;
        /** The image with update */
        image: string;
    };
}
/** A Stack was auto updated */
 | {
    type: "StackAutoUpdated";
    data: {
        /** The id of the stack */
        id: string;
        /** The name of the stack */
        name: string;
        /** The server id of server that the stack is on */
        server_id: string;
        /** The server name */
        server_name: string;
        /** One or more images that were updated */
        images: string[];
    };
}
/** An AWS builder failed to terminate. */
 | {
    type: "AwsBuilderTerminationFailed";
    data: {
        /** The id of the aws instance which failed to terminate */
        instance_id: string;
        /** A reason for the failure */
        message: string;
    };
}
/** A resource sync has pending updates */
 | {
    type: "ResourceSyncPendingUpdates";
    data: {
        /** The id of the resource sync */
        id: string;
        /** The name of the resource sync */
        name: string;
    };
}
/** A build has failed */
 | {
    type: "BuildFailed";
    data: {
        /** The id of the build */
        id: string;
        /** The name of the build */
        name: string;
        /** The version that failed to build */
        version: Version;
    };
}
/** A repo has failed */
 | {
    type: "RepoBuildFailed";
    data: {
        /** The id of the repo */
        id: string;
        /** The name of the repo */
        name: string;
    };
}
/** A procedure has failed */
 | {
    type: "ProcedureFailed";
    data: {
        /** The id of the procedure */
        id: string;
        /** The name of the procedure */
        name: string;
    };
}
/** An action has failed */
 | {
    type: "ActionFailed";
    data: {
        /** The id of the action */
        id: string;
        /** The name of the action */
        name: string;
    };
}
/** A schedule was run */
 | {
    type: "ScheduleRun";
    data: {
        /** Procedure or Action */
        resource_type: ResourceTarget["type"];
        /** The resource id */
        id: string;
        /** The resource name */
        name: string;
    };
}
/**
 * Custom header / body.
 * Produced using `/execute/SendAlert`
 */
 | {
    type: "Custom";
    data: {
        /** The alert message. */
        message: string;
        /** Message details. May be empty string. */
        details?: string;
    };
};
/** Representation of an alert in the system. */
export interface Alert {
    /**
     * The Mongo ID of the alert.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized Alert) }`
     */
    _id?: MongoId;
    /** Unix timestamp in milliseconds the alert was opened */
    ts: I64;
    /** Whether the alert is already resolved */
    resolved: boolean;
    /** The severity of the alert */
    level: SeverityLevel;
    /** The target of the alert */
    target: ResourceTarget;
    /** The data attached to the alert */
    data: AlertData;
    /** The timestamp of alert resolution */
    resolved_ts?: I64;
}
export type GetAlertResponse = Alert;
export type GetAlerterResponse = Alerter;
export interface BuildActionState {
    building: boolean;
}
export type GetBuildActionStateResponse = BuildActionState;
export type GetBuildResponse = Build;
export type GetBuilderResponse = Builder;
export type GetContainerLogResponse = Log;
export interface DeploymentActionState {
    pulling: boolean;
    deploying: boolean;
    starting: boolean;
    restarting: boolean;
    pausing: boolean;
    unpausing: boolean;
    stopping: boolean;
    destroying: boolean;
    renaming: boolean;
}
export type GetDeploymentActionStateResponse = DeploymentActionState;
export type GetDeploymentLogResponse = Log;
export type GetDeploymentResponse = Deployment;
export interface ContainerStats {
    name: string;
    cpu_perc: string;
    mem_perc: string;
    mem_usage: string;
    net_io: string;
    block_io: string;
    pids: string;
}
export type GetDeploymentStatsResponse = ContainerStats;
export type GetDockerRegistryAccountResponse = DockerRegistryAccount;
export type GetGitProviderAccountResponse = GitProviderAccount;
export declare enum Timelength {
    /** `1-sec` */
    OneSecond = "1-sec",
    /** `1-sec` */
    TwoSeconds = "2-sec",
    /** `1-sec` */
    ThreeSeconds = "3-sec",
    /** `5-sec` */
    FiveSeconds = "5-sec",
    /** `10-sec` */
    TenSeconds = "10-sec",
    /** `15-sec` */
    FifteenSeconds = "15-sec",
    /** `30-sec` */
    ThirtySeconds = "30-sec",
    /** `1-min` */
    OneMinute = "1-min",
    /** `2-min` */
    TwoMinutes = "2-min",
    /** `3-min` */
    ThreeMinutes = "3-min",
    /** `5-min` */
    FiveMinutes = "5-min",
    /** `10-min` */
    TenMinutes = "10-min",
    /** `15-min` */
    FifteenMinutes = "15-min",
    /** `30-min` */
    ThirtyMinutes = "30-min",
    /** `1-hr` */
    OneHour = "1-hr",
    /** `2-hr` */
    TwoHours = "2-hr",
    /** `3-hr` */
    ThreeHours = "3-hr",
    /** `6-hr` */
    SixHours = "6-hr",
    /** `8-hr` */
    EightHours = "8-hr",
    /** `12-hr` */
    TwelveHours = "12-hr",
    /** `1-day` */
    OneDay = "1-day",
    /** `2-day` */
    TwoDays = "2-day",
    /** `3-day` */
    ThreeDays = "3-day",
    /** `1-wk` */
    OneWeek = "1-wk",
    /** `2-wk` */
    TwoWeeks = "2-wk",
    /** `30-day` */
    ThirtyDays = "30-day"
}
/** Info about Periphery configuration */
export interface PeripheryInformation {
    /** The Periphery version. */
    version: string;
    /** The public key of Periphery */
    public_key: string;
    /** Whether terminals are disabled on this Periphery server */
    terminals_disabled: boolean;
    /** Whether container exec is disabled on this Periphery server */
    container_terminals_disabled: boolean;
    /** The rate the system stats are being polled from the system */
    stats_polling_rate: Timelength;
    /** Whether Periphery is successfully connected to docker daemon. */
    docker_connected: boolean;
    /** The host public ip, if it can be resolved. */
    public_ip?: string;
}
export type GetPeripheryInformationResponse = PeripheryInformation;
export type GetPermissionResponse = PermissionLevelAndSpecifics;
export interface ProcedureActionState {
    running: boolean;
}
export type GetProcedureActionStateResponse = ProcedureActionState;
export type GetProcedureResponse = Procedure;
export interface RepoActionState {
    /** Whether Repo currently cloning on the attached Server */
    cloning: boolean;
    /** Whether Repo currently pulling on the attached Server */
    pulling: boolean;
    /** Whether Repo currently building using the attached Builder. */
    building: boolean;
    /** Whether Repo currently renaming. */
    renaming: boolean;
}
export type GetRepoActionStateResponse = RepoActionState;
export interface RepoConfig {
    /** The server to clone the repo on. */
    server_id?: string;
    /** Attach a builder to 'build' the repo. */
    builder_id?: string;
    /** The git provider domain. Default: github.com */
    git_provider: string;
    /**
     * Whether to use https to clone the repo (versus http). Default: true
     *
     * Note. Komodo does not currently support cloning repos via ssh.
     */
    git_https: boolean;
    /**
     * The git account used to access private repos.
     * Passing empty string can only clone public repos.
     *
     * Note. A token for the account must be available in the core config or the builder server's periphery config
     * for the configured git provider.
     */
    git_account?: string;
    /** The github repo to clone. */
    repo?: string;
    /** The repo branch. */
    branch: string;
    /** Optionally set a specific commit hash. */
    commit?: string;
    /**
     * Explicitly specify the folder to clone the repo in.
     * - If absolute (has leading '/')
     * - Used directly as the path
     * - If relative
     * - Taken relative to Periphery `repo_dir` (ie `${root_directory}/repos`)
     */
    path?: string;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this repo.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
    /**
     * Command to be run after the repo is cloned.
     * The path is relative to the root of the repo.
     */
    on_clone?: SystemCommand;
    /**
     * Command to be run after the repo is pulled.
     * The path is relative to the root of the repo.
     */
    on_pull?: SystemCommand;
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
    /**
     * The environment variables passed to the compose file.
     * They will be written to path defined in env_file_path,
     * which is given relative to the run directory.
     *
     * If it is empty, no file will be written.
     */
    environment?: string;
    /**
     * The name of the written environment file before `docker compose up`.
     * Relative to the repo root.
     * Default: .env
     */
    env_file_path: string;
    /** Whether to skip secret interpolation into the repo environment variable file. */
    skip_secret_interp?: boolean;
}
export interface RepoInfo {
    /** When repo was last pulled */
    last_pulled_at?: I64;
    /** When repo was last built */
    last_built_at?: I64;
    /** Latest built short commit hash, or null. */
    built_hash?: string;
    /** Latest built commit message, or null. Only for repo based stacks */
    built_message?: string;
    /** Latest remote short commit hash, or null. */
    latest_hash?: string;
    /** Latest remote commit message, or null */
    latest_message?: string;
}
export type Repo = Resource<RepoConfig, RepoInfo>;
export type GetRepoResponse = Repo;
export interface ResourceSyncActionState {
    /** Whether sync currently syncing */
    syncing: boolean;
}
export type GetResourceSyncActionStateResponse = ResourceSyncActionState;
/** The sync configuration. */
export interface ResourceSyncConfig {
    /** Choose a Komodo Repo (Resource) to source the sync files. */
    linked_repo?: string;
    /** The git provider domain. Default: github.com */
    git_provider: string;
    /**
     * Whether to use https to clone the repo (versus http). Default: true
     *
     * Note. Komodo does not currently support cloning repos via ssh.
     */
    git_https: boolean;
    /** The Github repo used as the source of the build. */
    repo?: string;
    /** The branch of the repo. */
    branch: string;
    /** Optionally set a specific commit hash. */
    commit?: string;
    /**
     * The git account used to access private repos.
     * Passing empty string can only clone public repos.
     *
     * Note. A token for the account must be available in the core config or the builder server's periphery config
     * for the configured git provider.
     */
    git_account?: string;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this sync.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
    /**
     * Files are available on the Komodo Core host.
     * Specify the file / folder with [ResourceSyncConfig::resource_path].
     */
    files_on_host?: boolean;
    /**
     * The path of the resource file(s) to sync.
     * - If Files on Host, this is relative to the configured `sync_directory` in core config.
     * - If Git Repo based, this is relative to the root of the repo.
     * Can be a specific file, or a directory containing multiple files / folders.
     * See [https://komo.do/docs/sync-resources](https://komo.do/docs/sync-resources) for more information.
     */
    resource_path?: string[];
    /**
     * Enable "pushes" to the file,
     * which exports resources matching tags to single file.
     * - If using `files_on_host`, it is stored in the file_contents, which must point to a .toml file path (it will be created if it doesn't exist).
     * - If using `file_contents`, it is stored in the database.
     * When using this, "delete" mode is always enabled.
     */
    managed?: boolean;
    /**
     * Whether sync should delete resources
     * not declared in the resource files
     */
    delete?: boolean;
    /**
     * Whether sync should include resources.
     * Default: true
     */
    include_resources: boolean;
    /**
     * When using `managed` resource sync, will only export resources
     * matching all of the given tags. If none, will match all resources.
     */
    match_tags?: string[];
    /** Whether sync should include variables. */
    include_variables?: boolean;
    /** Whether sync should include user groups. */
    include_user_groups?: boolean;
    /**
     * Whether sync should send alert when it enters Pending state.
     * Default: true
     */
    pending_alert: boolean;
    /** Manage the file contents in the UI. */
    file_contents?: string;
}
export type DiffData = 
/** Resource will be created */
{
    type: "Create";
    data: {
        /** The name of resource to create */
        name?: string;
        /** The proposed resource to create in TOML */
        proposed: string;
    };
} | {
    type: "Update";
    data: {
        /** The proposed TOML */
        proposed: string;
        /** The current TOML */
        current: string;
    };
} | {
    type: "Delete";
    data: {
        /** The current TOML of the resource to delete */
        current: string;
    };
};
export interface ResourceDiff {
    /**
     * The resource target.
     * The target id will be empty if "Create" ResourceDiffType.
     */
    target: ResourceTarget;
    /** The data associated with the diff. */
    data: DiffData;
}
export interface SyncDeployUpdate {
    /** Resources to deploy */
    to_deploy: number;
    /** A readable log of all the changes to be applied */
    log: string;
}
export interface SyncFileContents {
    /** The base resource path. */
    resource_path?: string;
    /** The path of the file / error path relative to the resource path. */
    path: string;
    /** The contents of the file */
    contents: string;
}
export interface ResourceSyncInfo {
    /** Unix timestamp of last applied sync */
    last_sync_ts?: I64;
    /** Short commit hash of last applied sync */
    last_sync_hash?: string;
    /** Commit message of last applied sync */
    last_sync_message?: string;
    /** The list of pending updates to resources */
    resource_updates?: ResourceDiff[];
    /** The list of pending updates to variables */
    variable_updates?: DiffData[];
    /** The list of pending updates to user groups */
    user_group_updates?: DiffData[];
    /** The list of pending deploys to resources. */
    pending_deploy?: SyncDeployUpdate;
    /** If there is an error, it will be stored here */
    pending_error?: string;
    /** The commit hash which produced these pending updates. */
    pending_hash?: string;
    /** The commit message which produced these pending updates. */
    pending_message?: string;
    /** The current sync files */
    remote_contents?: SyncFileContents[];
    /** Any read errors in files by path */
    remote_errors?: SyncFileContents[];
}
export type ResourceSync = Resource<ResourceSyncConfig, ResourceSyncInfo>;
export type GetResourceSyncResponse = ResourceSync;
/** Current pending actions on the server. */
export interface ServerActionState {
    /** Server currently pruning networks */
    pruning_networks: boolean;
    /** Server currently pruning containers */
    pruning_containers: boolean;
    /** Server currently pruning images */
    pruning_images: boolean;
    /** Server currently pruning volumes */
    pruning_volumes: boolean;
    /** Server currently pruning docker builders */
    pruning_builders: boolean;
    /** Server currently pruning builx cache */
    pruning_buildx: boolean;
    /** Server currently pruning system */
    pruning_system: boolean;
    /** Server currently starting containers. */
    starting_containers: boolean;
    /** Server currently restarting containers. */
    restarting_containers: boolean;
    /** Server currently pausing containers. */
    pausing_containers: boolean;
    /** Server currently unpausing containers. */
    unpausing_containers: boolean;
    /** Server currently stopping containers. */
    stopping_containers: boolean;
}
export type GetServerActionStateResponse = ServerActionState;
/** Server configuration. */
export interface ServerConfig {
    /**
     * The ws/s address of the periphery client.
     * If unset, Server expects Periphery -> Core connection.
     */
    address?: string;
    /**
     * Only relevant for Core -> Periphery connections.
     * Whether to skip Periphery tls certificate validation.
     * This defaults to true because Periphery generates self-signed certificates by default,
     * but if you use valid certs you can switch this to false.
     */
    insecure_tls: boolean;
    /**
     * The address to use with links for containers on the server.
     * If empty, will use the 'address' for links.
     */
    external_address?: string;
    /** An optional region label */
    region?: string;
    /**
     * Whether a server is enabled.
     * If a server is disabled,
     * you won't be able to perform any actions on it or see deployment's status.
     * Default: false
     */
    enabled: boolean;
    /**
     * Whether to automatically rotate Server keys when
     * RotateAllServerKeys is called.
     * Default: true
     */
    auto_rotate_keys: boolean;
    /**
     * Deprecated. Use private / public keys instead.
     * An optional override passkey to use
     * to authenticate with periphery agent.
     * If this is empty, will use passkey in core config.
     */
    passkey?: string;
    /**
     * Sometimes the system stats reports a mount path that is not desired.
     * Use this field to filter it out from the report.
     */
    ignore_mounts?: string[];
    /**
     * Whether to trigger 'docker image prune -a -f' every 24 hours.
     * default: true
     */
    auto_prune: boolean;
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
    /**
     * Whether to monitor any server stats beyond passing health check.
     * default: true
     */
    stats_monitoring: boolean;
    /** Whether to send alerts about the servers reachability */
    send_unreachable_alerts: boolean;
    /** Whether to send alerts about the servers CPU status */
    send_cpu_alerts: boolean;
    /** Whether to send alerts about the servers MEM status */
    send_mem_alerts: boolean;
    /** Whether to send alerts about the servers DISK status */
    send_disk_alerts: boolean;
    /** Whether to send alerts about the servers version mismatch with core */
    send_version_mismatch_alerts: boolean;
    /** The percentage threshhold which triggers WARNING state for CPU. */
    cpu_warning: number;
    /** The percentage threshhold which triggers CRITICAL state for CPU. */
    cpu_critical: number;
    /** The percentage threshhold which triggers WARNING state for MEM. */
    mem_warning: number;
    /** The percentage threshhold which triggers CRITICAL state for MEM. */
    mem_critical: number;
    /** The percentage threshhold which triggers WARNING state for DISK. */
    disk_warning: number;
    /** The percentage threshhold which triggers CRITICAL state for DISK. */
    disk_critical: number;
    /** Scheduled maintenance windows during which alerts will be suppressed. */
    maintenance_windows?: MaintenanceWindow[];
}
export interface ServerInfo {
    /**
     * If a Periphery fails to authenticate to Core
     * for a disconnected server with invalid Periphery public key,
     * it will be stored here to accept the connection later on.
     */
    attempted_public_key?: string;
    /**
     * The expected public key associated with
     * private key of the periphery agent.
     */
    public_key?: string;
}
export type Server = Resource<ServerConfig, ServerInfo>;
export type GetServerResponse = Server;
export interface StackActionState {
    pulling: boolean;
    deploying: boolean;
    starting: boolean;
    restarting: boolean;
    pausing: boolean;
    unpausing: boolean;
    stopping: boolean;
    destroying: boolean;
}
export type GetStackActionStateResponse = StackActionState;
export type GetStackLogResponse = Log;
/**
 * Additional env file configuration for Stack.
 * Supports backward compatibility with string-only format.
 */
export interface AdditionalEnvFile {
    /** File path relative to run directory */
    path: string;
    /**
     * Whether Komodo should track this file's contents.
     * If true (default), Komodo will read, display, diff, and validate.
     * If false, only passed to docker compose via --env-file.
     * Useful for externally managed files (e.g., sops decrypted files).
     */
    track: boolean;
}
export declare enum StackFileRequires {
    /** Diff requires service redeploy. */
    Redeploy = "Redeploy",
    /** Diff requires service restart */
    Restart = "Restart",
    /** Diff requires no action. Default. */
    None = "None"
}
/** Configure additional file dependencies of the Stack. */
export interface StackFileDependency {
    /** Specify the file */
    path: string;
    /** Specify specific service/s */
    services?: string[];
    /** Specify */
    requires?: StackFileRequires;
}
/** The compose file configuration. */
export interface StackConfig {
    /**
     * The Swarm to deploy the Stack on, setting the Stack into Swarm mode.
     *
     * Note. If both swarm_id and server_id are set,
     * swarm_id overrides server_id and the Stack will be in Swarm mode.
     */
    swarm_id?: string;
    /**
     * The Server to deploy the Stack on, setting the Stack into Compose mode.
     *
     * Note. If both swarm_id and server_id are set,
     * swarm_id overrides server_id and the Stack will be in Swarm mode.
     */
    server_id?: string;
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
    /**
     * Optionally specify a custom project name for the stack.
     * If this is empty string, it will default to the stack name.
     * Used with `docker compose -p {project_name}` / `docker stack deploy {project_name}`.
     *
     * Note. Can be used to import pre-existing stacks with names that do not match Stack name.
     */
    project_name?: string;
    /**
     * Whether to automatically `compose pull` before redeploying stack.
     * Ensured latest images are deployed.
     * Will fail if the compose file specifies a locally build image.
     *
     * Note. Not used in Swarm mode.
     */
    auto_pull: boolean;
    /**
     * Whether to `docker compose build` before `compose down` / `compose up`.
     * Combine with build_extra_args for custom behaviors.
     *
     * Note. Not used in Swarm mode.
     */
    run_build?: boolean;
    /** Whether to poll for any updates to the images. */
    poll_for_updates?: boolean;
    /**
     * Whether to automatically redeploy when
     * newer images are found. Will implicitly
     * enable `poll_for_updates`, you don't need to
     * enable both.
     */
    auto_update?: boolean;
    /**
     * If auto update is enabled, Komodo will
     * by default only update the specific services
     * with image updates. If this parameter is set to true,
     * Komodo will redeploy the whole Stack (all services).
     */
    auto_update_all_services?: boolean;
    /** Whether to run `docker compose down` before `compose up`. */
    destroy_before_deploy?: boolean;
    /** Whether to skip secret interpolation into the stack environment variables. */
    skip_secret_interp?: boolean;
    /** Choose a Komodo Repo (Resource) to source the compose files. */
    linked_repo?: string;
    /** The git provider domain. Default: github.com */
    git_provider: string;
    /**
     * Whether to use https to clone the repo (versus http). Default: true
     *
     * Note. Komodo does not currently support cloning repos via ssh.
     */
    git_https: boolean;
    /**
     * The git account used to access private repos.
     * Passing empty string can only clone public repos.
     *
     * Note. A token for the account must be available in the core config or the builder server's periphery config
     * for the configured git provider.
     */
    git_account?: string;
    /**
     * The repo used as the source of the build.
     * {namespace}/{repo_name}
     */
    repo?: string;
    /** The branch of the repo. */
    branch: string;
    /** Optionally set a specific commit hash. */
    commit?: string;
    /** Optionally set a specific clone path */
    clone_path?: string;
    /**
     * By default, the Stack will `git pull` the repo after it is first cloned.
     * If this option is enabled, the repo folder will be deleted and recloned instead.
     */
    reclone?: boolean;
    /** Whether incoming webhooks actually trigger action. */
    webhook_enabled: boolean;
    /**
     * Optionally provide an alternate webhook secret for this stack.
     * If its an empty string, use the default secret from the config.
     */
    webhook_secret?: string;
    /**
     * By default, the Stack will `DeployStackIfChanged`.
     * If this option is enabled, will always run `DeployStack` without diffing.
     */
    webhook_force_deploy?: boolean;
    /**
     * If this is checked, the stack will source the files on the host.
     * Use `run_directory` and `file_paths` to specify the path on the host.
     * This is useful for those who wish to setup their files on the host,
     * rather than defining the contents in UI or in a git repo.
     */
    files_on_host?: boolean;
    /** Directory to change to (`cd`) before running `docker compose up -d`. */
    run_directory?: string;
    /**
     * Add paths to compose files, relative to the run path.
     * If this is empty, will use file `compose.yaml`.
     */
    file_paths?: string[];
    /**
     * The name of the written environment file before `docker compose up`.
     * Relative to the run directory root.
     * Default: .env
     *
     * Note. Not used in Swarm mode.
     */
    env_file_path: string;
    /**
     * Add additional env files to attach with `--env-file`.
     * Relative to the run directory root.
     *
     * Note. It is already included as an `additional_file`.
     * Don't add it again there.
     */
    additional_env_files?: AdditionalEnvFile[];
    /**
     * Add additional config files either in repo or on host to track.
     * Can add any files associated with the stack to enable editing them in the UI.
     * Doing so will also include diffing these when deciding to deploy in `DeployStackIfChanged`.
     * Relative to the run directory.
     *
     * Note. If the config file is .env and should be included in compose command
     * using `--env-file`, add it to `additional_env_files` instead.
     */
    config_files?: StackFileDependency[];
    /** Whether to send StackStateChange alerts for this stack. */
    send_alerts: boolean;
    /** Used with `registry_account` to login to a registry before docker compose up. */
    registry_provider?: string;
    /** Used with `registry_provider` to login to a registry before docker compose up. */
    registry_account?: string;
    /** The optional command to run before the Stack is deployed. */
    pre_deploy?: SystemCommand;
    /** The optional command to run after the Stack is deployed. */
    post_deploy?: SystemCommand;
    /**
     * The extra arguments to pass to the deploy command.
     *
     * - For Compose stack, uses `docker compose up -d [EXTRA_ARGS]`.
     * - For Swarm mode. `docker stack deploy [EXTRA_ARGS] STACK_NAME`
     *
     * If empty, no extra arguments will be passed.
     */
    extra_args?: string[];
    /**
     * The extra arguments to pass after `docker compose build`.
     * If empty, no extra build arguments will be passed.
     * Only used if `run_build: true`
     *
     * Note. Not used in Swarm mode.
     */
    build_extra_args?: string[];
    /**
     * Optional command wrapper for secrets management tools.
     * Wraps the docker compose up command with a prefix command.
     * Use [[COMPOSE_COMMAND]] as placeholder for the full compose command.
     *
     * Examples:
     * - "op run -- [[COMPOSE_COMMAND]]" (1password CLI)
     * - "sops exec-file --no-fifo /path/to/secret.env '[[COMPOSE_COMMAND]]'" (sops)
     */
    compose_cmd_wrapper?: string;
    /**
     * Ignore certain services declared in the compose file when checking
     * the stack status. For example, an init service might be exited, but the
     * stack should be healthy. This init service should be in `ignore_services`
     */
    ignore_services?: string[];
    /**
     * The contents of the file directly, for management in the UI.
     * If this is empty, it will fall back to checking git config for
     * repo based compose file.
     * Supports variable / secret interpolation.
     */
    file_contents?: string;
    /**
     * The environment variables passed to the compose file.
     * They will be written to path defined in env_file_path,
     * which is given relative to the run directory.
     *
     * If it is empty, no file will be written.
     *
     * Note. Not used in Swarm mode.
     */
    environment?: string;
}
export interface FileContents {
    /** The path to the file */
    path: string;
    /** The contents of the file */
    contents: string;
}
export interface StackServiceNames {
    /** The name of the service */
    service_name: string;
    /**
     * Will either be the declared container_name in the compose file,
     * or a pattern to match auto named containers.
     *
     * Auto named containers are composed of three parts:
     *
     * 1. The name of the compose project (top level name field of compose file).
     * This defaults to the name of the parent folder of the compose file.
     * Komodo will always set it to be the name of the stack, but imported stacks
     * will have a different name.
     * 2. The service name
     * 3. The replica number
     *
     * Example: stacko-mongo-1.
     *
     * This stores only 1. and 2., ie stacko-mongo.
     * Containers will be matched via regex like `^container_name-?[0-9]*$``
     *
     * Note. Setting container_name is not supported by Swarm,
     * so will always be 1. and 2. in Swarm mode.
     */
    container_name: string;
    /** The services image. */
    image?: string;
}
/**
 * Same as [FileContents] with some extra
 * info specific to Stacks.
 */
export interface StackRemoteFileContents {
    /** The path to the file */
    path: string;
    /** The contents of the file */
    contents: string;
    /**
     * The services depending on this file,
     * or empty for global requirement (eg all compose files and env files).
     */
    services?: string[];
    /** Whether diff requires Redeploy / Restart / None */
    requires?: StackFileRequires;
}
export interface StackInfo {
    /**
     * If any of the expected compose / additional files are missing in the repo,
     * they will be stored here.
     */
    missing_files?: string[];
    /**
     * The deployed project name.
     * This is updated whenever Komodo successfully deploys the stack.
     * If it is present, Komodo will use it for actions over other options,
     * to ensure control is maintained after changing the project name (there is no rename compose project api).
     */
    deployed_project_name?: string;
    /** Deployed short commit hash, or null. Only for repo based stacks. */
    deployed_hash?: string;
    /** Deployed commit message, or null. Only for repo based stacks */
    deployed_message?: string;
    /**
     * The deployed compose / additional file contents.
     * This is updated whenever Komodo successfully deploys the stack.
     */
    deployed_contents?: FileContents[];
    /**
     * The deployed service names.
     * This is updated whenever it is empty, or deployed contents is updated.
     */
    deployed_services?: StackServiceNames[];
    /**
     * The output of `docker compose config`.
     * This is updated whenever Komodo successfully deploys the stack.
     */
    deployed_config?: string;
    /**
     * The latest service names.
     * This is updated whenever the stack cache refreshes, using the latest file contents (either db defined or remote).
     */
    latest_services?: StackServiceNames[];
    /**
     * The remote compose / additional file contents, whether on host or in repo.
     * This is updated whenever Komodo refreshes the stack cache.
     * It will be empty if the file is defined directly in the stack config.
     */
    remote_contents?: StackRemoteFileContents[];
    /** If there was an error in getting the remote contents, it will be here. */
    remote_errors?: FileContents[];
    /** Latest commit hash, or null */
    latest_hash?: string;
    /** Latest commit message, or null */
    latest_message?: string;
}
export type Stack = Resource<StackConfig, StackInfo>;
export type GetStackResponse = Stack;
export interface SwarmActionState {
}
export type GetSwarmActionStateResponse = SwarmActionState;
export interface SwarmConfig {
    /**
     * The Servers which are swarm manager nodes.
     * If a Server is not reachable or gives error,
     * tries the next Server.
     */
    server_ids?: string[];
    /** Configure quick links that are displayed in the resource header */
    links?: string[];
}
export interface SwarmInfo {
}
export type Swarm = Resource<SwarmConfig, SwarmInfo>;
export type GetSwarmResponse = Swarm;
export type GetSwarmServiceLogResponse = Log;
/** System information of a server */
export interface SystemInformation {
    /** The system name */
    name?: string;
    /** The system long os version */
    os?: string;
    /** System's kernel version */
    kernel?: string;
    /** Physical core count */
    core_count?: number;
    /** System hostname based off DNS */
    host_name?: string;
    /** The CPU's brand */
    cpu_brand: string;
}
export type GetSystemInformationResponse = SystemInformation;
export interface SystemLoadAverage {
    /** 1m load average */
    one: number;
    /** 5m load average */
    five: number;
    /** 15m load average */
    fifteen: number;
}
/** Info for a single disk mounted on the system. */
export interface SingleDiskUsage {
    /** The mount point of the disk */
    mount: string;
    /** Detected file system */
    file_system: string;
    /** Used portion of the disk in GB */
    used_gb: number;
    /** Total size of the disk in GB */
    total_gb: number;
}
/** Realtime system stats data. */
export interface SystemStats {
    /** Cpu usage percentage */
    cpu_perc: number;
    /** Load average (1m, 5m, 15m) */
    load_average?: SystemLoadAverage;
    /**
     * [1.15.9+]
     * Free memory in GB.
     * This is really the 'Free' memory, not the 'Available' memory.
     * It may be different than mem_total_gb - mem_used_gb.
     */
    mem_free_gb?: number;
    /** Used memory in GB. 'Total' - 'Available' (not free) memory. */
    mem_used_gb: number;
    /** Total memory in GB */
    mem_total_gb: number;
    /** Breakdown of individual disks, ie their usages, sizes, and mount points */
    disks: SingleDiskUsage[];
    /** Network ingress usage in MB */
    network_ingress_bytes?: number;
    /** Network egress usage in MB */
    network_egress_bytes?: number;
    /** The rate the system stats are being polled from the system */
    polling_rate: Timelength;
    /** Unix timestamp in milliseconds when stats were last polled */
    refresh_ts: I64;
    /** Unix timestamp in milliseconds when disk list was last refreshed */
    refresh_list_ts: I64;
}
export type GetSystemStatsResponse = SystemStats;
export declare enum TagColor {
    LightSlate = "LightSlate",
    Slate = "Slate",
    DarkSlate = "DarkSlate",
    LightRed = "LightRed",
    Red = "Red",
    DarkRed = "DarkRed",
    LightOrange = "LightOrange",
    Orange = "Orange",
    DarkOrange = "DarkOrange",
    LightAmber = "LightAmber",
    Amber = "Amber",
    DarkAmber = "DarkAmber",
    LightYellow = "LightYellow",
    Yellow = "Yellow",
    DarkYellow = "DarkYellow",
    LightLime = "LightLime",
    Lime = "Lime",
    DarkLime = "DarkLime",
    LightGreen = "LightGreen",
    Green = "Green",
    DarkGreen = "DarkGreen",
    LightEmerald = "LightEmerald",
    Emerald = "Emerald",
    DarkEmerald = "DarkEmerald",
    LightTeal = "LightTeal",
    Teal = "Teal",
    DarkTeal = "DarkTeal",
    LightCyan = "LightCyan",
    Cyan = "Cyan",
    DarkCyan = "DarkCyan",
    LightSky = "LightSky",
    Sky = "Sky",
    DarkSky = "DarkSky",
    LightBlue = "LightBlue",
    Blue = "Blue",
    DarkBlue = "DarkBlue",
    LightIndigo = "LightIndigo",
    Indigo = "Indigo",
    DarkIndigo = "DarkIndigo",
    LightViolet = "LightViolet",
    Violet = "Violet",
    DarkViolet = "DarkViolet",
    LightPurple = "LightPurple",
    Purple = "Purple",
    DarkPurple = "DarkPurple",
    LightFuchsia = "LightFuchsia",
    Fuchsia = "Fuchsia",
    DarkFuchsia = "DarkFuchsia",
    LightPink = "LightPink",
    Pink = "Pink",
    DarkPink = "DarkPink",
    LightRose = "LightRose",
    Rose = "Rose",
    DarkRose = "DarkRose"
}
export interface Tag {
    /**
     * The Mongo ID of the tag.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized Tag) }`
     */
    _id?: MongoId;
    name: string;
    owner?: string;
    /** Hex color code with alpha for UI display */
    color?: TagColor;
}
export type GetTagResponse = Tag;
export type GetUpdateResponse = Update;
/**
 * Permission users at the group level.
 *
 * All users that are part of a group inherit the group's permissions.
 * A user can be a part of multiple groups. A user's permission on a particular resource
 * will be resolved to be the maximum permission level between the user's own permissions and
 * any groups they are a part of.
 */
export interface UserGroup {
    /**
     * The Mongo ID of the UserGroup.
     * This field is de/serialized from/to JSON as
     * `{ "_id": { "$oid": "..." }, ...(rest of serialized User) }`
     */
    _id?: MongoId;
    /** A name for the user group */
    name: string;
    /** Whether all users will implicitly have the permissions in this group. */
    everyone?: boolean;
    /** User ids of group members */
    users?: string[];
    /** Give the user group elevated permissions on all resources of a certain type */
    all?: Record<ResourceTarget["type"], PermissionLevelAndSpecifics | PermissionLevel>;
    /** Unix time (ms) when user group last updated */
    updated_at?: I64;
}
export type GetUserGroupResponse = UserGroup;
export type GetUserResponse = User;
export type GetVariableResponse = Variable;
export declare enum ContainerStateStatusEnum {
    Running = "running",
    Created = "created",
    Paused = "paused",
    Restarting = "restarting",
    Exited = "exited",
    Removing = "removing",
    Dead = "dead",
    Empty = ""
}
export declare enum HealthStatusEnum {
    Empty = "",
    None = "none",
    Starting = "starting",
    Healthy = "healthy",
    Unhealthy = "unhealthy"
}
/** HealthcheckResult stores information about a single run of a healthcheck probe */
export interface HealthcheckResult {
    /** Date and time at which this check started in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    Start?: string;
    /** Date and time at which this check ended in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    End?: string;
    /** ExitCode meanings:  - `0` healthy - `1` unhealthy - `2` reserved (considered unhealthy) - other values: error running probe */
    ExitCode?: I64;
    /** Output from last check */
    Output?: string;
}
/** Health stores information about the container's healthcheck results. */
export interface ContainerHealth {
    /** Status is one of `none`, `starting`, `healthy` or `unhealthy`  - \"none\"      Indicates there is no healthcheck - \"starting\"  Starting indicates that the container is not yet ready - \"healthy\"   Healthy indicates that the container is running correctly - \"unhealthy\" Unhealthy indicates that the container has a problem */
    Status?: HealthStatusEnum;
    /** FailingStreak is the number of consecutive failures */
    FailingStreak?: I64;
    /** Log contains the last few results (oldest first) */
    Log?: HealthcheckResult[];
}
/** ContainerState stores container's running state. It's part of ContainerJSONBase and will be returned by the \"inspect\" command. */
export interface ContainerState {
    /** String representation of the container state. Can be one of \"created\", \"running\", \"paused\", \"restarting\", \"removing\", \"exited\", or \"dead\". */
    Status?: ContainerStateStatusEnum;
    /** Whether this container is running.  Note that a running container can be _paused_. The `Running` and `Paused` booleans are not mutually exclusive:  When pausing a container (on Linux), the freezer cgroup is used to suspend all processes in the container. Freezing the process requires the process to be running. As a result, paused containers are both `Running` _and_ `Paused`.  Use the `Status` field instead to determine if a container's state is \"running\". */
    Running?: boolean;
    /** Whether this container is paused. */
    Paused?: boolean;
    /** Whether this container is restarting. */
    Restarting?: boolean;
    /** Whether a process within this container has been killed because it ran out of memory since the container was last started. */
    OOMKilled?: boolean;
    Dead?: boolean;
    /** The process ID of this container */
    Pid?: I64;
    /** The last exit code of this container */
    ExitCode?: I64;
    Error?: string;
    /** The time when this container was last started. */
    StartedAt?: string;
    /** The time when this container last exited. */
    FinishedAt?: string;
    Health?: ContainerHealth;
}
export type Usize = number;
export interface ResourcesBlkioWeightDevice {
    Path?: string;
    Weight?: Usize;
}
export interface ThrottleDevice {
    /** Device path */
    Path?: string;
    /** Rate */
    Rate?: I64;
}
/** A device mapping between the host and container */
export interface DeviceMapping {
    PathOnHost?: string;
    PathInContainer?: string;
    CgroupPermissions?: string;
}
/** A request for devices to be sent to device drivers */
export interface DeviceRequest {
    Driver?: string;
    Count?: I64;
    DeviceIDs?: string[];
    /** A list of capabilities; an OR list of AND lists of capabilities. */
    Capabilities?: string[][];
    /** Driver-specific options, specified as a key/value pairs. These options are passed directly to the driver. */
    Options?: Record<string, string>;
}
export interface ResourcesUlimits {
    /** Name of ulimit */
    Name?: string;
    /** Soft limit */
    Soft?: I64;
    /** Hard limit */
    Hard?: I64;
}
/** The logging configuration for this container */
export interface HostConfigLogConfig {
    Type?: string;
    Config?: Record<string, string>;
}
/** PortBinding represents a binding between a host IP address and a host port. */
export interface PortBinding {
    /** Host IP address that the container's port is mapped to. */
    HostIp?: string;
    /** Host port number that the container's port is mapped to. */
    HostPort?: string;
}
export declare enum RestartPolicyNameEnum {
    Empty = "",
    No = "no",
    Always = "always",
    UnlessStopped = "unless-stopped",
    OnFailure = "on-failure"
}
/** The behavior to apply when the container exits. The default is not to restart.  An ever increasing delay (double the previous delay, starting at 100ms) is added before each restart to prevent flooding the server. */
export interface RestartPolicy {
    /** - Empty string means not to restart - `no` Do not automatically restart - `always` Always restart - `unless-stopped` Restart always except when the user has manually stopped the container - `on-failure` Restart only when the container exit code is non-zero */
    Name?: RestartPolicyNameEnum;
    /** If `on-failure` is used, the number of times to retry before giving up. */
    MaximumRetryCount?: I64;
}
export declare enum MountTypeEnum {
    Empty = "",
    Bind = "bind",
    Volume = "volume",
    Image = "image",
    Tmpfs = "tmpfs",
    Npipe = "npipe",
    Cluster = "cluster"
}
export declare enum MountBindOptionsPropagationEnum {
    Empty = "",
    Private = "private",
    Rprivate = "rprivate",
    Shared = "shared",
    Rshared = "rshared",
    Slave = "slave",
    Rslave = "rslave"
}
/** Optional configuration for the `bind` type. */
export interface MountBindOptions {
    /** A propagation mode with the value `[r]private`, `[r]shared`, or `[r]slave`. */
    Propagation?: MountBindOptionsPropagationEnum;
    /** Disable recursive bind mount. */
    NonRecursive?: boolean;
    /** Create mount point on host if missing */
    CreateMountpoint?: boolean;
    /** Make the mount non-recursively read-only, but still leave the mount recursive (unless NonRecursive is set to `true` in conjunction).  Addded in v1.44, before that version all read-only mounts were non-recursive by default. To match the previous behaviour this will default to `true` for clients on versions prior to v1.44. */
    ReadOnlyNonRecursive?: boolean;
    /** Raise an error if the mount cannot be made recursively read-only. */
    ReadOnlyForceRecursive?: boolean;
}
/** Map of driver specific options */
export interface MountVolumeOptionsDriverConfig {
    /** Name of the driver to use to create the volume. */
    Name?: string;
    /** key/value map of driver specific options. */
    Options?: Record<string, string>;
}
/** Optional configuration for the `volume` type. */
export interface MountVolumeOptions {
    /** Populate volume with data from the target. */
    NoCopy?: boolean;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    DriverConfig?: MountVolumeOptionsDriverConfig;
    /** Source path inside the volume. Must be relative without any back traversals. */
    Subpath?: string;
}
/** Optional configuration for the `tmpfs` type. */
export interface MountTmpfsOptions {
    /** The size for the tmpfs mount in bytes. */
    SizeBytes?: I64;
    /** The permission mode for the tmpfs mount in an integer. */
    Mode?: I64;
}
export interface Mount {
    /** Container path. */
    Target?: string;
    /** Mount source (e.g. a volume name, a host path). */
    Source?: string;
    /**
     * The mount type. Available types:
     * - `bind` Mounts a file or directory from the host into the container. Must exist prior to creating the container.
     * - `volume` Creates a volume with the given name and options (or uses a pre-existing volume with the same name and options). These are **not** removed when the container is removed.
     * - `tmpfs` Create a tmpfs with the given options. The mount source cannot be specified for tmpfs. - `npipe` Mounts a named pipe from the host into the container. Must exist prior to creating the container.
     * - `cluster` a Swarm cluster volume
     */
    Type?: MountTypeEnum;
    /** Whether the mount should be read-only. */
    ReadOnly?: boolean;
    /** The consistency requirement for the mount: `default`, `consistent`, `cached`, or `delegated`. */
    Consistency?: string;
    BindOptions?: MountBindOptions;
    VolumeOptions?: MountVolumeOptions;
    TmpfsOptions?: MountTmpfsOptions;
}
export declare enum HostConfigCgroupnsModeEnum {
    Empty = "",
    Private = "private",
    Host = "host"
}
export declare enum HostConfigIsolationEnum {
    Empty = "",
    Default = "default",
    Process = "process",
    Hyperv = "hyperv"
}
/** Container configuration that depends on the host we are running on */
export interface HostConfig {
    /** An integer value representing this container's relative CPU weight versus other containers. */
    CpuShares?: I64;
    /** Memory limit in bytes. */
    Memory?: I64;
    /** Path to `cgroups` under which the container's `cgroup` is created. If the path is not absolute, the path is considered to be relative to the `cgroups` path of the init process. Cgroups are created if they do not already exist. */
    CgroupParent?: string;
    /** Block IO weight (relative weight). */
    BlkioWeight?: number;
    /** Block IO weight (relative device weight) in the form:  ``` [{\"Path\": \"device_path\", \"Weight\": weight}] ``` */
    BlkioWeightDevice?: ResourcesBlkioWeightDevice[];
    /** Limit read rate (bytes per second) from a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ``` */
    BlkioDeviceReadBps?: ThrottleDevice[];
    /** Limit write rate (bytes per second) to a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ``` */
    BlkioDeviceWriteBps?: ThrottleDevice[];
    /** Limit read rate (IO per second) from a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ``` */
    BlkioDeviceReadIOps?: ThrottleDevice[];
    /** Limit write rate (IO per second) to a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ``` */
    BlkioDeviceWriteIOps?: ThrottleDevice[];
    /** The length of a CPU period in microseconds. */
    CpuPeriod?: I64;
    /** Microseconds of CPU time that the container can get in a CPU period. */
    CpuQuota?: I64;
    /** The length of a CPU real-time period in microseconds. Set to 0 to allocate no time allocated to real-time tasks. */
    CpuRealtimePeriod?: I64;
    /** The length of a CPU real-time runtime in microseconds. Set to 0 to allocate no time allocated to real-time tasks. */
    CpuRealtimeRuntime?: I64;
    /** CPUs in which to allow execution (e.g., `0-3`, `0,1`). */
    CpusetCpus?: string;
    /** Memory nodes (MEMs) in which to allow execution (0-3, 0,1). Only effective on NUMA systems. */
    CpusetMems?: string;
    /** A list of devices to add to the container. */
    Devices?: DeviceMapping[];
    /** a list of cgroup rules to apply to the container */
    DeviceCgroupRules?: string[];
    /** A list of requests for devices to be sent to device drivers. */
    DeviceRequests?: DeviceRequest[];
    /** Hard limit for kernel TCP buffer memory (in bytes). Depending on the OCI runtime in use, this option may be ignored. It is no longer supported by the default (runc) runtime.  This field is omitted when empty. */
    KernelMemoryTCP?: I64;
    /** Memory soft limit in bytes. */
    MemoryReservation?: I64;
    /** Total memory limit (memory + swap). Set as `-1` to enable unlimited swap. */
    MemorySwap?: I64;
    /** Tune a container's memory swappiness behavior. Accepts an integer between 0 and 100. */
    MemorySwappiness?: I64;
    /** CPU quota in units of 10<sup>-9</sup> CPUs. */
    NanoCpus?: I64;
    /** Disable OOM Killer for the container. */
    OomKillDisable?: boolean;
    /** Run an init inside the container that forwards signals and reaps processes. This field is omitted if empty, and the default (as configured on the daemon) is used. */
    Init?: boolean;
    /** Tune a container's PIDs limit. Set `0` or `-1` for unlimited, or `null` to not change. */
    PidsLimit?: I64;
    /** A list of resource limits to set in the container. For example:  ``` {\"Name\": \"nofile\", \"Soft\": 1024, \"Hard\": 2048} ``` */
    Ulimits?: ResourcesUlimits[];
    /** The number of usable CPUs (Windows only).  On Windows Server containers, the processor resource controls are mutually exclusive. The order of precedence is `CPUCount` first, then `CPUShares`, and `CPUPercent` last. */
    CpuCount?: I64;
    /** The usable percentage of the available CPUs (Windows only).  On Windows Server containers, the processor resource controls are mutually exclusive. The order of precedence is `CPUCount` first, then `CPUShares`, and `CPUPercent` last. */
    CpuPercent?: I64;
    /** Maximum IOps for the container system drive (Windows only) */
    IOMaximumIOps?: I64;
    /** Maximum IO in bytes per second for the container system drive (Windows only). */
    IOMaximumBandwidth?: I64;
    /** A list of volume bindings for this container. Each volume binding is a string in one of these forms:  - `host-src:container-dest[:options]` to bind-mount a host path   into the container. Both `host-src`, and `container-dest` must   be an _absolute_ path. - `volume-name:container-dest[:options]` to bind-mount a volume   managed by a volume driver into the container. `container-dest`   must be an _absolute_ path.  `options` is an optional, comma-delimited list of:  - `nocopy` disables automatic copying of data from the container   path to the volume. The `nocopy` flag only applies to named volumes. - `[ro|rw]` mounts a volume read-only or read-write, respectively.   If omitted or set to `rw`, volumes are mounted read-write. - `[z|Z]` applies SELinux labels to allow or deny multiple containers   to read and write to the same volume.     - `z`: a _shared_ content label is applied to the content. This       label indicates that multiple containers can share the volume       content, for both reading and writing.     - `Z`: a _private unshared_ label is applied to the content.       This label indicates that only the current container can use       a private volume. Labeling systems such as SELinux require       proper labels to be placed on volume content that is mounted       into a container. Without a label, the security system can       prevent a container's processes from using the content. By       default, the labels set by the host operating system are not       modified. - `[[r]shared|[r]slave|[r]private]` specifies mount   [propagation behavior](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt).   This only applies to bind-mounted volumes, not internal volumes   or named volumes. Mount propagation requires the source mount   point (the location where the source directory is mounted in the   host operating system) to have the correct propagation properties.   For shared volumes, the source mount point must be set to `shared`.   For slave volumes, the mount must be set to either `shared` or   `slave`. */
    Binds?: string[];
    /** Path to a file where the container ID is written */
    ContainerIDFile?: string;
    LogConfig?: HostConfigLogConfig;
    /** Network mode to use for this container. Supported standard values are: `bridge`, `host`, `none`, and `container:<name|id>`. Any other value is taken as a custom network's name to which this container should connect to. */
    NetworkMode?: string;
    PortBindings?: Record<string, PortBinding[]>;
    RestartPolicy?: RestartPolicy;
    /** Automatically remove the container when the container's process exits. This has no effect if `RestartPolicy` is set. */
    AutoRemove?: boolean;
    /** Driver that this container uses to mount volumes. */
    VolumeDriver?: string;
    /** A list of volumes to inherit from another container, specified in the form `<container name>[:<ro|rw>]`. */
    VolumesFrom?: string[];
    /** Specification for mounts to be added to the container. */
    Mounts?: Mount[];
    /** Initial console size, as an `[height, width]` array. */
    ConsoleSize?: number[];
    /** Arbitrary non-identifying metadata attached to container and provided to the runtime when the container is started. */
    Annotations?: Record<string, string>;
    /** A list of kernel capabilities to add to the container. Conflicts with option 'Capabilities'. */
    CapAdd?: string[];
    /** A list of kernel capabilities to drop from the container. Conflicts with option 'Capabilities'. */
    CapDrop?: string[];
    /** cgroup namespace mode for the container. Possible values are:  - `\"private\"`: the container runs in its own private cgroup namespace - `\"host\"`: use the host system's cgroup namespace  If not specified, the daemon default is used, which can either be `\"private\"` or `\"host\"`, depending on daemon version, kernel support and configuration. */
    CgroupnsMode?: HostConfigCgroupnsModeEnum;
    /** A list of DNS servers for the container to use. */
    Dns?: string[];
    /** A list of DNS options. */
    DnsOptions?: string[];
    /** A list of DNS search domains. */
    DnsSearch?: string[];
    /** A list of hostnames/IP mappings to add to the container's `/etc/hosts` file. Specified in the form `[\"hostname:IP\"]`. */
    ExtraHosts?: string[];
    /** A list of additional groups that the container process will run as. */
    GroupAdd?: string[];
    /** IPC sharing mode for the container. Possible values are:  - `\"none\"`: own private IPC namespace, with /dev/shm not mounted - `\"private\"`: own private IPC namespace - `\"shareable\"`: own private IPC namespace, with a possibility to share it with other containers - `\"container:<name|id>\"`: join another (shareable) container's IPC namespace - `\"host\"`: use the host system's IPC namespace  If not specified, daemon default is used, which can either be `\"private\"` or `\"shareable\"`, depending on daemon version and configuration. */
    IpcMode?: string;
    /** Cgroup to use for the container. */
    Cgroup?: string;
    /** A list of links for the container in the form `container_name:alias`. */
    Links?: string[];
    /** An integer value containing the score given to the container in order to tune OOM killer preferences. */
    OomScoreAdj?: I64;
    /** Set the PID (Process) Namespace mode for the container. It can be either:  - `\"container:<name|id>\"`: joins another container's PID namespace - `\"host\"`: use the host's PID namespace inside the container */
    PidMode?: string;
    /** Gives the container full access to the host. */
    Privileged?: boolean;
    /** Allocates an ephemeral host port for all of a container's exposed ports.  Ports are de-allocated when the container stops and allocated when the container starts. The allocated port might be changed when restarting the container.  The port is selected from the ephemeral port range that depends on the kernel. For example, on Linux the range is defined by `/proc/sys/net/ipv4/ip_local_port_range`. */
    PublishAllPorts?: boolean;
    /** Mount the container's root filesystem as read only. */
    ReadonlyRootfs?: boolean;
    /** A list of string values to customize labels for MLS systems, such as SELinux. */
    SecurityOpt?: string[];
    /** Storage driver options for this container, in the form `{\"size\": \"120G\"}`. */
    StorageOpt?: Record<string, string>;
    /** A map of container directories which should be replaced by tmpfs mounts, and their corresponding mount options. For example:  ``` { \"/run\": \"rw,noexec,nosuid,size=65536k\" } ``` */
    Tmpfs?: Record<string, string>;
    /** UTS namespace to use for the container. */
    UTSMode?: string;
    /** Sets the usernamespace mode for the container when usernamespace remapping option is enabled. */
    UsernsMode?: string;
    /** Size of `/dev/shm` in bytes. If omitted, the system uses 64MB. */
    ShmSize?: I64;
    /** A list of kernel parameters (sysctls) to set in the container. For example:  ``` {\"net.ipv4.ip_forward\": \"1\"} ``` */
    Sysctls?: Record<string, string>;
    /** Runtime to use with this container. */
    Runtime?: string;
    /** Isolation technology of the container. (Windows only) */
    Isolation?: HostConfigIsolationEnum;
    /** The list of paths to be masked inside the container (this overrides the default set of paths). */
    MaskedPaths?: string[];
    /** The list of paths to be set as read-only inside the container (this overrides the default set of paths). */
    ReadonlyPaths?: string[];
}
/** Information about the storage driver used to store the container's and image's filesystem. */
export interface GraphDriverData {
    /** Name of the storage driver. */
    Name?: string;
    /** Low-level storage metadata, provided as key/value pairs.  This information is driver-specific, and depends on the storage-driver in use, and should be used for informational purposes only. */
    Data?: Record<string, string>;
}
/** MountPoint represents a mount point configuration inside the container. This is used for reporting the mountpoints in use by a container. */
export interface MountPoint {
    /** The mount type:  - `bind` a mount of a file or directory from the host into the container. - `volume` a docker volume with the given `Name`. - `tmpfs` a `tmpfs`. - `npipe` a named pipe from the host into the container. - `cluster` a Swarm cluster volume */
    Type?: MountTypeEnum;
    /** Name is the name reference to the underlying data defined by `Source` e.g., the volume name. */
    Name?: string;
    /** Source location of the mount.  For volumes, this contains the storage location of the volume (within `/var/lib/docker/volumes/`). For bind-mounts, and `npipe`, this contains the source (host) part of the bind-mount. For `tmpfs` mount points, this field is empty. */
    Source?: string;
    /** Destination is the path relative to the container root (`/`) where the `Source` is mounted inside the container. */
    Destination?: string;
    /** Driver is the volume driver used to create the volume (if it is a volume). */
    Driver?: string;
    /** Mode is a comma separated list of options supplied by the user when creating the bind/volume mount.  The default is platform-specific (`\"z\"` on Linux, empty on Windows). */
    Mode?: string;
    /** Whether the mount is mounted writable (read-write). */
    RW?: boolean;
    /** Propagation describes how mounts are propagated from the host into the mount point, and vice-versa. Refer to the [Linux kernel documentation](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt) for details. This field is not used on Windows. */
    Propagation?: string;
}
/** A test to perform to check that the container is healthy. */
export interface HealthConfig {
    /** The test to perform. Possible values are:  - `[]` inherit healthcheck from image or parent image - `[\"NONE\"]` disable healthcheck - `[\"CMD\", args...]` exec arguments directly - `[\"CMD-SHELL\", command]` run command with system's default shell */
    Test?: string[];
    /** The time to wait between checks in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit. */
    Interval?: I64;
    /** The time to wait before considering the check to have hung. It should be 0 or at least 1000000 (1 ms). 0 means inherit. */
    Timeout?: I64;
    /** The number of consecutive failures needed to consider a container as unhealthy. 0 means inherit. */
    Retries?: I64;
    /** Start period for the container to initialize before starting health-retries countdown in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit. */
    StartPeriod?: I64;
    /** The time to wait between checks in nanoseconds during the start period. It should be 0 or at least 1000000 (1 ms). 0 means inherit. */
    StartInterval?: I64;
}
/** Configuration for a container that is portable between hosts.  When used as `ContainerConfig` field in an image, `ContainerConfig` is an optional field containing the configuration of the container that was last committed when creating the image.  Previous versions of Docker builder used this field to store build cache, and it is not in active use anymore. */
export interface ContainerConfig {
    /** The hostname to use for the container, as a valid RFC 1123 hostname. */
    Hostname?: string;
    /** The domain name to use for the container. */
    Domainname?: string;
    /** The user that commands are run as inside the container. */
    User?: string;
    /** Whether to attach to `stdin`. */
    AttachStdin?: boolean;
    /** Whether to attach to `stdout`. */
    AttachStdout?: boolean;
    /** Whether to attach to `stderr`. */
    AttachStderr?: boolean;
    /** An object mapping ports to an empty object in the form:  `{\"<port>/<tcp|udp|sctp>\": {}}` */
    ExposedPorts?: Record<string, Record<string, undefined>>;
    /** Attach standard streams to a TTY, including `stdin` if it is not closed. */
    Tty?: boolean;
    /** Open `stdin` */
    OpenStdin?: boolean;
    /** Close `stdin` after one attached client disconnects */
    StdinOnce?: boolean;
    /** A list of environment variables to set inside the container in the form `[\"VAR=value\", ...]`. A variable without `=` is removed from the environment, rather than to have an empty value. */
    Env?: string[];
    /** Command to run specified as a string or an array of strings. */
    Cmd?: string[];
    Healthcheck?: HealthConfig;
    /** Command is already escaped (Windows only) */
    ArgsEscaped?: boolean;
    /** The name (or reference) of the image to use when creating the container, or which was used when the container was created. */
    Image?: string;
    /** An object mapping mount point paths inside the container to empty objects. */
    Volumes?: Record<string, Record<string, undefined>>;
    /** The working directory for commands to run in. */
    WorkingDir?: string;
    /** The entry point for the container as a string or an array of strings.  If the array consists of exactly one empty string (`[\"\"]`) then the entry point is reset to system default (i.e., the entry point used by docker when there is no `ENTRYPOINT` instruction in the `Dockerfile`). */
    Entrypoint?: string[];
    /** Disable networking for the container. */
    NetworkDisabled?: boolean;
    /** MAC address of the container.  Deprecated: this field is deprecated in API v1.44 and up. Use EndpointSettings.MacAddress instead. */
    MacAddress?: string;
    /** `ONBUILD` metadata that were defined in the image's `Dockerfile`. */
    OnBuild?: string[];
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    /** Signal to stop a container as a string or unsigned integer. */
    StopSignal?: string;
    /** Timeout to stop a container in seconds. */
    StopTimeout?: I64;
    /** Shell for when `RUN`, `CMD`, and `ENTRYPOINT` uses a shell. */
    Shell?: string[];
}
/** EndpointIPAMConfig represents an endpoint's IPAM configuration. */
export interface EndpointIpamConfig {
    IPv4Address?: string;
    IPv6Address?: string;
    LinkLocalIPs?: string[];
}
/** Configuration for a network endpoint. */
export interface EndpointSettings {
    IPAMConfig?: EndpointIpamConfig;
    Links?: string[];
    /** MAC address for the endpoint on this network. The network driver might ignore this parameter. */
    MacAddress?: string;
    Aliases?: string[];
    /** Unique ID of the network. */
    NetworkID?: string;
    /** Unique ID for the service endpoint in a Sandbox. */
    EndpointID?: string;
    /** Gateway address for this network. */
    Gateway?: string;
    /** IPv4 address. */
    IPAddress?: string;
    /** Mask length of the IPv4 address. */
    IPPrefixLen?: I64;
    /** IPv6 gateway address. */
    IPv6Gateway?: string;
    /** Global IPv6 address. */
    GlobalIPv6Address?: string;
    /** Mask length of the global IPv6 address. */
    GlobalIPv6PrefixLen?: I64;
    /** DriverOpts is a mapping of driver options and values. These options are passed directly to the driver and are driver specific. */
    DriverOpts?: Record<string, string>;
    /** List of all DNS names an endpoint has on a specific network. This list is based on the container name, network aliases, container short ID, and hostname.  These DNS names are non-fully qualified but can contain several dots. You can get fully qualified DNS names by appending `.<network-name>`. For instance, if container name is `my.ctr` and the network is named `testnet`, `DNSNames` will contain `my.ctr` and the FQDN will be `my.ctr.testnet`. */
    DNSNames?: string[];
}
/** NetworkSettings exposes the network settings in the API */
export interface NetworkSettings {
    /** Name of the default bridge interface when dockerd's --bridge flag is set. */
    Bridge?: string;
    /** SandboxID uniquely represents a container's network stack. */
    SandboxID?: string;
    Ports?: Record<string, PortBinding[]>;
    /** SandboxKey is the full path of the netns handle */
    SandboxKey?: string;
    /** Information about all networks that the container is connected to. */
    Networks?: Record<string, EndpointSettings>;
}
export interface Container {
    /** The ID of the container */
    Id?: string;
    /** The time the container was created */
    Created?: string;
    /** The path to the command being run */
    Path?: string;
    /** The arguments to the command being run */
    Args?: string[];
    State?: ContainerState;
    /** The container's image ID */
    Image?: string;
    ResolvConfPath?: string;
    HostnamePath?: string;
    HostsPath?: string;
    LogPath?: string;
    Name?: string;
    RestartCount?: I64;
    Driver?: string;
    Platform?: string;
    MountLabel?: string;
    ProcessLabel?: string;
    AppArmorProfile?: string;
    /** IDs of exec instances that are running in the container. */
    ExecIDs?: string[];
    HostConfig?: HostConfig;
    GraphDriver?: GraphDriverData;
    /** The size of files that have been created or changed by this container. */
    SizeRw?: I64;
    /** The total size of all the files in this container. */
    SizeRootFs?: I64;
    Mounts?: MountPoint[];
    Config?: ContainerConfig;
    NetworkSettings?: NetworkSettings;
}
export type InspectDeploymentContainerResponse = Container;
export type InspectDockerContainerResponse = Container;
/** Information about the image's RootFS, including the layer IDs. */
export interface ImageInspectRootFs {
    Type?: string;
    Layers?: string[];
}
/** Additional metadata of the image in the local cache. This information is local to the daemon, and not part of the image itself. */
export interface ImageInspectMetadata {
    /** Date and time at which the image was last tagged in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if the image was tagged locally, and omitted otherwise. */
    LastTagTime?: string;
}
/** Information about an image in the local image cache. */
export interface Image {
    /** ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image. */
    Id?: string;
    /** List of image names/tags in the local image cache that reference this image.  Multiple image tags can refer to the same image, and this list may be empty if no tags reference the image, in which case the image is \"untagged\", in which case it can still be referenced by its ID. */
    RepoTags?: string[];
    /** List of content-addressable digests of locally available image manifests that the image is referenced from. Multiple manifests can refer to the same image.  These digests are usually only available if the image was either pulled from a registry, or if the image was pushed to a registry, which is when the manifest is generated and its digest calculated. */
    RepoDigests?: string[];
    /** ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry. */
    Parent?: string;
    /** Optional message that was set when committing or importing the image. */
    Comment?: string;
    /** Date and time at which the image was created, formatted in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if present in the image, and omitted otherwise. */
    Created?: string;
    /** The version of Docker that was used to build the image.  Depending on how the image was created, this field may be empty. */
    DockerVersion?: string;
    /** Name of the author that was specified when committing the image, or as specified through MAINTAINER (deprecated) in the Dockerfile. */
    Author?: string;
    /** Configuration for a container that is portable between hosts. */
    Config?: ContainerConfig;
    /** Hardware CPU architecture that the image runs on. */
    Architecture?: string;
    /** CPU architecture variant (presently ARM-only). */
    Variant?: string;
    /** Operating System the image is built to run on. */
    Os?: string;
    /** Operating System version the image is built to run on (especially for Windows). */
    OsVersion?: string;
    /** Total size of the image including all layers it is composed of. */
    Size?: I64;
    GraphDriver?: GraphDriverData;
    RootFS?: ImageInspectRootFs;
    Metadata?: ImageInspectMetadata;
}
export type InspectDockerImageResponse = Image;
export interface IpamConfig {
    Subnet?: string;
    IPRange?: string;
    Gateway?: string;
    AuxiliaryAddresses: Record<string, string>;
}
export interface Ipam {
    /** Name of the IPAM driver to use. */
    Driver?: string;
    /** List of IPAM configuration options, specified as a map:  ``` {\"Subnet\": <CIDR>, \"IPRange\": <CIDR>, \"Gateway\": <IP address>, \"AuxAddress\": <device_name:IP address>} ``` */
    Config: IpamConfig[];
    /** Driver-specific options, specified as a map. */
    Options: Record<string, string>;
}
export interface NetworkContainer {
    /** This is the key on the incoming map of NetworkContainer */
    ContainerID?: string;
    Name?: string;
    EndpointID?: string;
    MacAddress?: string;
    IPv4Address?: string;
    IPv6Address?: string;
}
export interface Network {
    Name?: string;
    Id?: string;
    Created?: string;
    Scope?: string;
    Driver?: string;
    EnableIPv6?: boolean;
    IPAM?: Ipam;
    Internal?: boolean;
    Attachable?: boolean;
    Ingress?: boolean;
    /** This field is turned from map into array for easier usability. */
    Containers: NetworkContainer[];
    Options?: Record<string, string>;
    Labels?: Record<string, string>;
}
export type InspectDockerNetworkResponse = Network;
export declare enum VolumeScopeEnum {
    Empty = "",
    Local = "local",
    Global = "global"
}
export type U64 = number;
/** The version number of the object such as node, service, etc. This is needed to avoid conflicting writes. The client must send the version number along with the modified specification when updating these objects.  This approach ensures safe concurrency and determinism in that the change on the object may not be applied if the version number has changed from the last read. In other words, if two update requests specify the same base version, only one of the requests can succeed. As a result, two separate update requests that happen at the same time will not unintentionally overwrite each other. */
export interface ObjectVersion {
    Index?: U64;
}
export declare enum ClusterVolumeSpecAccessModeScopeEnum {
    Empty = "",
    Single = "single",
    Multi = "multi"
}
export declare enum ClusterVolumeSpecAccessModeSharingEnum {
    Empty = "",
    None = "none",
    Readonly = "readonly",
    Onewriter = "onewriter",
    All = "all"
}
/** One cluster volume secret entry. Defines a key-value pair that is passed to the plugin. */
export interface ClusterVolumeSpecAccessModeSecrets {
    /** Key is the name of the key of the key-value pair passed to the plugin. */
    Key?: string;
    /** Secret is the swarm Secret object from which to read data. This can be a Secret name or ID. The Secret data is retrieved by swarm and used as the value of the key-value pair passed to the plugin. */
    Secret?: string;
}
export type Topology = Record<string, PortBinding[]>;
/** Requirements for the accessible topology of the volume. These fields are optional. For an in-depth description of what these fields mean, see the CSI specification. */
export interface ClusterVolumeSpecAccessModeAccessibilityRequirements {
    /** A list of required topologies, at least one of which the volume must be accessible from. */
    Requisite?: Topology[];
    /** A list of topologies that the volume should attempt to be provisioned in. */
    Preferred?: Topology[];
}
/** The desired capacity that the volume should be created with. If empty, the plugin will decide the capacity. */
export interface ClusterVolumeSpecAccessModeCapacityRange {
    /** The volume must be at least this big. The value of 0 indicates an unspecified minimum */
    RequiredBytes?: I64;
    /** The volume must not be bigger than this. The value of 0 indicates an unspecified maximum. */
    LimitBytes?: I64;
}
export declare enum ClusterVolumeSpecAccessModeAvailabilityEnum {
    Empty = "",
    Active = "active",
    Pause = "pause",
    Drain = "drain"
}
/** Defines how the volume is used by tasks. */
export interface ClusterVolumeSpecAccessMode {
    /** The set of nodes this volume can be used on at one time. - `single` The volume may only be scheduled to one node at a time. - `multi` the volume may be scheduled to any supported number of nodes at a time. */
    Scope?: ClusterVolumeSpecAccessModeScopeEnum;
    /** The number and way that different tasks can use this volume at one time. - `none` The volume may only be used by one task at a time. - `readonly` The volume may be used by any number of tasks, but they all must mount the volume as readonly - `onewriter` The volume may be used by any number of tasks, but only one may mount it as read/write. - `all` The volume may have any number of readers and writers. */
    Sharing?: ClusterVolumeSpecAccessModeSharingEnum;
    /** Swarm Secrets that are passed to the CSI storage plugin when operating on this volume. */
    Secrets?: ClusterVolumeSpecAccessModeSecrets[];
    AccessibilityRequirements?: ClusterVolumeSpecAccessModeAccessibilityRequirements;
    CapacityRange?: ClusterVolumeSpecAccessModeCapacityRange;
    /** The availability of the volume for use in tasks. - `active` The volume is fully available for scheduling on the cluster - `pause` No new workloads should use the volume, but existing workloads are not stopped. - `drain` All workloads using this volume should be stopped and rescheduled, and no new ones should be started. */
    Availability?: ClusterVolumeSpecAccessModeAvailabilityEnum;
}
/** Cluster-specific options used to create the volume. */
export interface ClusterVolumeSpec {
    /** Group defines the volume group of this volume. Volumes belonging to the same group can be referred to by group name when creating Services.  Referring to a volume by group instructs Swarm to treat volumes in that group interchangeably for the purpose of scheduling. Volumes with an empty string for a group technically all belong to the same, emptystring group. */
    Group?: string;
    AccessMode?: ClusterVolumeSpecAccessMode;
}
/** Information about the global status of the volume. */
export interface ClusterVolumeInfo {
    /** The capacity of the volume in bytes. A value of 0 indicates that the capacity is unknown. */
    CapacityBytes?: I64;
    /** A map of strings to strings returned from the storage plugin when the volume is created. */
    VolumeContext?: Record<string, string>;
    /** The ID of the volume as returned by the CSI storage plugin. This is distinct from the volume's ID as provided by Docker. This ID is never used by the user when communicating with Docker to refer to this volume. If the ID is blank, then the Volume has not been successfully created in the plugin yet. */
    VolumeID?: string;
    /** The topology this volume is actually accessible from. */
    AccessibleTopology?: Topology[];
}
export declare enum ClusterVolumePublishStatusStateEnum {
    Empty = "",
    PendingPublish = "pending-publish",
    Published = "published",
    PendingNodeUnpublish = "pending-node-unpublish",
    PendingControllerUnpublish = "pending-controller-unpublish"
}
export interface ClusterVolumePublishStatus {
    /** The ID of the Swarm node the volume is published on. */
    NodeID?: string;
    /** The published state of the volume. * `pending-publish` The volume should be published to this node, but the call to the controller plugin to do so has not yet been successfully completed. * `published` The volume is published successfully to the node. * `pending-node-unpublish` The volume should be unpublished from the node, and the manager is awaiting confirmation from the worker that it has done so. * `pending-controller-unpublish` The volume is successfully unpublished from the node, but has not yet been successfully unpublished on the controller. */
    State?: ClusterVolumePublishStatusStateEnum;
    /** A map of strings to strings returned by the CSI controller plugin when a volume is published. */
    PublishContext?: Record<string, string>;
}
/** Options and information specific to, and only present on, Swarm CSI cluster volumes. */
export interface ClusterVolume {
    /** The Swarm ID of this volume. Because cluster volumes are Swarm objects, they have an ID, unlike non-cluster volumes. This ID can be used to refer to the Volume instead of the name. */
    ID?: string;
    Version?: ObjectVersion;
    CreatedAt?: string;
    UpdatedAt?: string;
    Spec?: ClusterVolumeSpec;
    Info?: ClusterVolumeInfo;
    /** The status of the volume as it pertains to its publishing and use on specific nodes */
    PublishStatus?: ClusterVolumePublishStatus[];
}
/** Usage details about the volume. This information is used by the `GET /system/df` endpoint, and omitted in other endpoints. */
export interface VolumeUsageData {
    /** Amount of disk space used by the volume (in bytes). This information is only available for volumes created with the `\"local\"` volume driver. For volumes created with other volume drivers, this field is set to `-1` (\"not available\") */
    Size: I64;
    /** The number of containers referencing this volume. This field is set to `-1` if the reference-count is not available. */
    RefCount: I64;
}
export interface Volume {
    /** Name of the volume. */
    Name: string;
    /** Name of the volume driver used by the volume. */
    Driver: string;
    /** Mount path of the volume on the host. */
    Mountpoint: string;
    /** Date/Time the volume was created. */
    CreatedAt?: string;
    /** Low-level details about the volume, provided by the volume driver. Details are returned as a map with key/value pairs: `{\"key\":\"value\",\"key2\":\"value2\"}`.  The `Status` field is optional, and is omitted if the volume driver does not support this feature. */
    Status?: Record<string, Record<string, undefined>>;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    /** The level at which the volume exists. Either `global` for cluster-wide, or `local` for machine level. */
    Scope?: VolumeScopeEnum;
    ClusterVolume?: ClusterVolume;
    /** The driver specific options used when creating the volume. */
    Options?: Record<string, string>;
    UsageData?: VolumeUsageData;
}
export type InspectDockerVolumeResponse = Volume;
export type InspectStackContainerResponse = Container;
/** Describes a permission the user has to accept upon installing the plugin. */
export interface PluginPrivilege {
    Name?: string;
    Description?: string;
    Value?: string[];
}
/**
 * Plugin spec for the service.
 * *(Experimental release only.)*
 * <p><br /></p>  > **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive. PluginSpec is only used when the Runtime field > is set to `plugin`. NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`.
 */
export interface TaskSpecPluginSpec {
    /** The name or 'alias' to use for the plugin. */
    Name?: string;
    /** The plugin image reference to use. */
    Remote?: string;
    /** Disable the plugin once scheduled. */
    Disabled?: boolean;
    PluginPrivilege?: PluginPrivilege[];
}
/** CredentialSpec for managed service account (Windows only) */
export interface TaskSpecContainerSpecPrivilegesCredentialSpec {
    /** Load credential spec from a Swarm Config with the given ID. The specified config must also be present in the Configs field with the Runtime property set.  <p><br /></p>   > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive. */
    Config?: string;
    /** Load credential spec from this file. The file is read by the daemon, and must be present in the `CredentialSpecs` subdirectory in the docker data directory, which defaults to `C:\\ProgramData\\Docker\\` on Windows.  For example, specifying `spec.json` loads `C:\\ProgramData\\Docker\\CredentialSpecs\\spec.json`.  <p><br /></p>  > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive. */
    File?: string;
    /** Load credential spec from this value in the Windows registry. The specified registry value must be located in:  `HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Virtualization\\Containers\\CredentialSpecs`  <p><br /></p>   > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive. */
    Registry?: string;
}
/** SELinux labels of the container */
export interface TaskSpecContainerSpecPrivilegesSeLinuxContext {
    /** Disable SELinux */
    Disable?: boolean;
    /** SELinux user label */
    User?: string;
    /** SELinux role label */
    Role?: string;
    /** SELinux type label */
    Type?: string;
    /** SELinux level label */
    Level?: string;
}
export declare enum TaskSpecContainerSpecPrivilegesSeccompModeEnum {
    EMPTY = "",
    DEFAULT = "default",
    UNCONFINED = "unconfined",
    CUSTOM = "custom"
}
/** Options for configuring seccomp on the container */
export interface TaskSpecContainerSpecPrivilegesSeccomp {
    Mode?: TaskSpecContainerSpecPrivilegesSeccompModeEnum;
    /** The custom seccomp profile as a json object */
    Profile?: string;
}
export declare enum TaskSpecContainerSpecPrivilegesAppArmorModeEnum {
    EMPTY = "",
    DEFAULT = "default",
    DISABLED = "disabled"
}
/** Options for configuring AppArmor on the container */
export interface TaskSpecContainerSpecPrivilegesAppArmor {
    Mode?: TaskSpecContainerSpecPrivilegesAppArmorModeEnum;
}
/** Security options for the container */
export interface TaskSpecContainerSpecPrivileges {
    CredentialSpec?: TaskSpecContainerSpecPrivilegesCredentialSpec;
    SELinuxContext?: TaskSpecContainerSpecPrivilegesSeLinuxContext;
    Seccomp?: TaskSpecContainerSpecPrivilegesSeccomp;
    AppArmor?: TaskSpecContainerSpecPrivilegesAppArmor;
    /** Configuration of the no_new_privs bit in the container */
    NoNewPrivileges?: boolean;
}
/** Specification for DNS related configurations in resolver configuration file (`resolv.conf`). */
export interface TaskSpecContainerSpecDnsConfig {
    /** The IP addresses of the name servers. */
    Nameservers?: string[];
    /** A search list for host-name lookup. */
    Search?: string[];
    /** A list of internal resolver variables to be modified (e.g., `debug`, `ndots:3`, etc.). */
    Options?: string[];
}
/** File represents a specific target that is backed by a file. */
export interface TaskSpecContainerSpecFile {
    /** Name represents the final filename in the filesystem. */
    Name?: string;
    /** UID represents the file UID. */
    UID?: string;
    /** GID represents the file GID. */
    GID?: string;
    /** Mode represents the FileMode of the file. */
    Mode?: number;
}
export interface TaskSpecContainerSpecSecrets {
    File?: TaskSpecContainerSpecFile;
    /** SecretID represents the ID of the specific secret that we're referencing. */
    SecretID?: string;
    /** SecretName is the name of the secret that this references, but this is just provided for lookup/display purposes. The secret in the reference will be identified by its ID. */
    SecretName?: string;
}
export interface TaskSpecContainerSpecConfigs {
    File?: TaskSpecContainerSpecFile;
    /** ConfigID represents the ID of the specific config that we're referencing. */
    ConfigID?: string;
    /** ConfigName is the name of the config that this references, but this is just provided for lookup/display purposes. The config in the reference will be identified by its ID. */
    ConfigName?: string;
}
export declare enum TaskSpecContainerSpecIsolationEnum {
    DEFAULT = "default",
    PROCESS = "process",
    HYPERV = "hyperv",
    EMPTY = ""
}
/**
 * Container spec for the service.
 * **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive.
 * PluginSpec is only used when the Runtime field > is set to `plugin`.
 * NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`.
 */
export interface TaskSpecContainerSpec {
    /** The image name to use for the container */
    Image?: string;
    /** User-defined key/value data. */
    Labels?: Record<string, string>;
    /** The command to be run in the image. */
    Command?: string[];
    /** Arguments to the command. */
    Args?: string[];
    /** The hostname to use for the container, as a valid [RFC 1123](https://tools.ietf.org/html/rfc1123) hostname. */
    Hostname?: string;
    /** A list of environment variables in the form `VAR=value`. */
    Env?: string[];
    /** The working directory for commands to run in. */
    Dir?: string;
    /** The user inside the container. */
    User?: string;
    /** A list of additional groups that the container process will run as. */
    Groups?: string[];
    Privileges?: TaskSpecContainerSpecPrivileges;
    /** Whether a pseudo-TTY should be allocated. */
    TTY?: boolean;
    /** Open `stdin` */
    OpenStdin?: boolean;
    /** Mount the container's root filesystem as read only. */
    ReadOnly?: boolean;
    /** Specification for mounts to be added to containers created as part of the service. */
    Mounts?: Mount[];
    /** Signal to stop the container. */
    StopSignal?: string;
    /** Amount of time to wait for the container to terminate before forcefully killing it. */
    StopGracePeriod?: I64;
    HealthCheck?: HealthConfig;
    /** A list of hostname/IP mappings to add to the container's `hosts` file. The format of extra hosts is specified in the [hosts(5)](http://man7.org/linux/man-pages/man5/hosts.5.html) man page:      IP_address canonical_hostname [aliases...] */
    Hosts?: string[];
    DNSConfig?: TaskSpecContainerSpecDnsConfig;
    /** Secrets contains references to zero or more secrets that will be exposed to the service. */
    Secrets?: TaskSpecContainerSpecSecrets[];
    /** An integer value containing the score given to the container in order to tune OOM killer preferences. */
    OomScoreAdj?: I64;
    /** Configs contains references to zero or more configs that will be exposed to the service. */
    Configs?: TaskSpecContainerSpecConfigs[];
    /** Isolation technology of the containers running the service. (Windows only) */
    Isolation?: TaskSpecContainerSpecIsolationEnum;
    /** Run an init inside the container that forwards signals and reaps processes. This field is omitted if empty, and the default (as configured on the daemon) is used. */
    Init?: boolean;
    /** Set kernel namedspaced parameters (sysctls) in the container. The Sysctls option on services accepts the same sysctls as the are supported on containers. Note that while the same sysctls are supported, no guarantees or checks are made about their suitability for a clustered environment, and it's up to the user to determine whether a given sysctl will work properly in a Service. */
    Sysctls?: Record<string, string>;
    /** A list of kernel capabilities to add to the default set for the container. */
    CapabilityAdd?: string[];
    /** A list of kernel capabilities to drop from the default set for the container. */
    CapabilityDrop?: string[];
    /** A list of resource limits to set in the container. For example: `{\"Name\": \"nofile\", \"Soft\": 1024, \"Hard\": 2048}`\" */
    Ulimits?: ResourcesUlimits[];
}
/** Read-only spec type for non-swarm containers attached to swarm overlay networks.  <p><br /></p>  > **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive. PluginSpec is only used when the Runtime field > is set to `plugin`. NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`. */
export interface TaskSpecNetworkAttachmentSpec {
    /** ID of the container represented by this task */
    ContainerID?: string;
}
/** An object describing a limit on resources which can be requested by a task. */
export interface Limit {
    NanoCPUs?: I64;
    MemoryBytes?: I64;
    /** Limits the maximum number of PIDs in the container. Set `0` for unlimited. */
    Pids?: I64;
}
export interface ResourceObject {
    NanoCPUs?: I64;
    MemoryBytes?: I64;
    GenericResources?: GenericResources;
}
/** Resource requirements which apply to each individual container created as part of the service. */
export interface TaskSpecResources {
    /** Define resources limits. */
    Limits?: Limit;
    /** Define resources reservation. */
    Reservations?: ResourceObject;
}
export declare enum TaskSpecRestartPolicyConditionEnum {
    EMPTY = "",
    NONE = "none",
    ON_FAILURE = "on-failure",
    ANY = "any"
}
/** Specification for the restart policy which applies to containers created as part of this service. */
export interface TaskSpecRestartPolicy {
    /** Condition for restart. */
    Condition?: TaskSpecRestartPolicyConditionEnum;
    /** Delay between restart attempts. */
    Delay?: I64;
    /** Maximum attempts to restart a given container before giving up (default value is 0, which is ignored). */
    MaxAttempts?: I64;
    /** Windows is the time window used to evaluate the restart policy (default value is 0, which is unbounded). */
    Window?: I64;
}
export interface TaskSpecPlacementSpread {
    /** label descriptor, such as `engine.labels.az`. */
    SpreadDescriptor?: string;
}
export interface TaskSpecPlacementPreferences {
    Spread?: TaskSpecPlacementSpread;
}
export interface Platform {
    /** Architecture represents the hardware architecture (for example, `x86_64`). */
    Architecture?: string;
    /** OS represents the Operating System (for example, `linux` or `windows`). */
    OS?: string;
}
export interface TaskSpecPlacement {
    /** An array of constraint expressions to limit the set of nodes where a task can be scheduled. Constraint expressions can either use a _match_ (`==`) or _exclude_ (`!=`) rule. Multiple constraints find nodes that satisfy every expression (AND match). Constraints can match node or Docker Engine labels as follows:  node attribute       | matches                        | example ---------------------|--------------------------------|----------------------------------------------- `node.id`            | Node ID                        | `node.id==2ivku8v2gvtg4` `node.hostname`      | Node hostname                  | `node.hostname!=node-2` `node.role`          | Node role (`manager`/`worker`) | `node.role==manager` `node.platform.os`   | Node operating system          | `node.platform.os==windows` `node.platform.arch` | Node architecture              | `node.platform.arch==x86_64` `node.labels`        | User-defined node labels       | `node.labels.security==high` `engine.labels`      | Docker Engine's labels         | `engine.labels.operatingsystem==ubuntu-24.04`  `engine.labels` apply to Docker Engine labels like operating system, drivers, etc. Swarm administrators add `node.labels` for operational purposes by using the [`node update endpoint`](#operation/NodeUpdate). */
    Constraints?: string[];
    /** Preferences provide a way to make the scheduler aware of factors such as topology. They are provided in order from highest to lowest precedence. */
    Preferences?: TaskSpecPlacementPreferences[];
    /** Maximum number of replicas for per node (default value is 0, which is unlimited) */
    MaxReplicas?: I64;
    /** Platforms stores all the platforms that the service's image can run on. This field is used in the platform filter for scheduling. If empty, then the platform filter is off, meaning there are no scheduling restrictions. */
    Platforms?: Platform[];
}
/** Specifies how a service should be attached to a particular network. */
export interface NetworkAttachmentConfig {
    /** The target network for attachment. Must be a network name or ID. */
    Target?: string;
    /** Discoverable alternate names for the service on this network. */
    Aliases?: string[];
    /** Driver attachment options for the network target. */
    DriverOpts?: Record<string, string>;
}
/**
 * Specifies the log driver to use for tasks created from this spec.
 * If not present, the default one for the swarm will be used,
 * finally falling back to the engine default if not specified.
 */
export interface TaskSpecLogDriver {
    Name?: string;
    Options?: Record<string, string>;
}
/** User modifiable task configuration. */
export interface TaskSpec {
    PluginSpec?: TaskSpecPluginSpec;
    ContainerSpec?: TaskSpecContainerSpec;
    NetworkAttachmentSpec?: TaskSpecNetworkAttachmentSpec;
    Resources?: TaskSpecResources;
    RestartPolicy?: TaskSpecRestartPolicy;
    Placement?: TaskSpecPlacement;
    /** A counter that triggers an update even if no relevant parameters have been changed. */
    ForceUpdate?: U64;
    /** Runtime is the type of runtime specified for the task executor. */
    Runtime?: string;
    /** Specifies which networks the service should attach to. */
    Networks?: NetworkAttachmentConfig[];
    LogDriver?: TaskSpecLogDriver;
}
export interface ServiceSpecModeReplicated {
    Replicas?: I64;
}
/** The mode used for services with a finite number of tasks that run to a completed state. */
export interface ServiceSpecModeReplicatedJob {
    /** The maximum number of replicas to run simultaneously. */
    MaxConcurrent?: I64;
    /** The total number of replicas desired to reach the Completed state. If unset, will default to the value of `MaxConcurrent` */
    TotalCompletions?: I64;
}
/** Scheduling mode for the service. */
export interface ServiceSpecMode {
    Replicated?: ServiceSpecModeReplicated;
    ReplicatedJob?: ServiceSpecModeReplicatedJob;
}
export declare enum ServiceSpecUpdateConfigFailureActionEnum {
    EMPTY = "",
    CONTINUE = "continue",
    PAUSE = "pause",
    ROLLBACK = "rollback"
}
export declare enum ServiceSpecUpdateConfigOrderEnum {
    EMPTY = "",
    STOP_FIRST = "stop-first",
    START_FIRST = "start-first"
}
/** Specification for the update strategy of the service. */
export interface ServiceSpecUpdateConfig {
    /** Maximum number of tasks to be updated in one iteration (0 means unlimited parallelism). */
    Parallelism?: I64;
    /** Amount of time between updates, in nanoseconds. */
    Delay?: I64;
    /** Action to take if an updated task fails to run, or stops running during the update. */
    FailureAction?: ServiceSpecUpdateConfigFailureActionEnum;
    /** Amount of time to monitor each updated task for failures, in nanoseconds. */
    Monitor?: I64;
    /** The fraction of tasks that may fail during an update before the failure action is invoked, specified as a floating point number between 0 and 1. */
    MaxFailureRatio?: number;
    /** The order of operations when rolling out an updated task. Either the old task is shut down before the new task is started, or the new task is started before the old task is shut down. */
    Order?: ServiceSpecUpdateConfigOrderEnum;
}
export declare enum ServiceSpecRollbackConfigFailureActionEnum {
    EMPTY = "",
    CONTINUE = "continue",
    PAUSE = "pause"
}
export declare enum ServiceSpecRollbackConfigOrderEnum {
    EMPTY = "",
    STOP_FIRST = "stop-first",
    START_FIRST = "start-first"
}
/** Specification for the rollback strategy of the service. */
export interface ServiceSpecRollbackConfig {
    /** Maximum number of tasks to be rolled back in one iteration (0 means unlimited parallelism). */
    Parallelism?: I64;
    /** Amount of time between rollback iterations, in nanoseconds. */
    Delay?: I64;
    /** Action to take if an rolled back task fails to run, or stops running during the rollback. */
    FailureAction?: ServiceSpecRollbackConfigFailureActionEnum;
    /** Amount of time to monitor each rolled back task for failures, in nanoseconds. */
    Monitor?: I64;
    /** The fraction of tasks that may fail during a rollback before the failure action is invoked, specified as a floating point number between 0 and 1. */
    MaxFailureRatio?: number;
    /** The order of operations when rolling back a task. Either the old task is shut down before the new task is started, or the new task is started before the old task is shut down. */
    Order?: ServiceSpecRollbackConfigOrderEnum;
}
export declare enum EndpointSpecModeEnum {
    EMPTY = "",
    VIP = "vip",
    DNSRR = "dnsrr"
}
export declare enum EndpointPortConfigProtocolEnum {
    EMPTY = "",
    TCP = "tcp",
    UDP = "udp",
    SCTP = "sctp"
}
export declare enum EndpointPortConfigPublishModeEnum {
    EMPTY = "",
    INGRESS = "ingress",
    HOST = "host"
}
export interface EndpointPortConfig {
    Name?: string;
    Protocol?: EndpointPortConfigProtocolEnum;
    /** The port inside the container. */
    TargetPort?: I64;
    /** The port on the swarm hosts. */
    PublishedPort?: I64;
    /** The mode in which port is published.  <p><br /></p>  - \"ingress\" makes the target port accessible on every node,   regardless of whether there is a task for the service running on   that node or not. - \"host\" bypasses the routing mesh and publish the port directly on   the swarm node where that service is running. */
    PublishMode?: EndpointPortConfigPublishModeEnum;
}
/** Properties that can be configured to access and load balance a service. */
export interface EndpointSpec {
    /** The mode of resolution to use for internal load balancing between tasks. */
    Mode?: EndpointSpecModeEnum;
    /** List of exposed ports that this service is accessible on from the outside. Ports can only be provided if `vip` resolution mode is used. */
    Ports?: EndpointPortConfig[];
}
/** User modifiable configuration for a service. */
export interface ServiceSpec {
    /** Name of the service. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    TaskTemplate?: TaskSpec;
    Mode?: ServiceSpecMode;
    UpdateConfig?: ServiceSpecUpdateConfig;
    RollbackConfig?: ServiceSpecRollbackConfig;
    /** Specifies which networks the service should attach to.  Deprecated: This field is deprecated since v1.44. The Networks field in TaskSpec should be used instead. */
    Networks?: NetworkAttachmentConfig[];
    EndpointSpec?: EndpointSpec;
}
export interface ServiceEndpointVirtualIps {
    NetworkID?: string;
    Addr?: string;
}
export interface ServiceEndpoint {
    Spec?: EndpointSpec;
    Ports?: EndpointPortConfig[];
    VirtualIPs?: ServiceEndpointVirtualIps[];
}
export declare enum ServiceUpdateStatusStateEnum {
    EMPTY = "",
    UPDATING = "updating",
    PAUSED = "paused",
    COMPLETED = "completed",
    ROLLBACK_STARTED = "rollback_started",
    ROLLBACK_PAUSED = "rollback_paused",
    ROLLBACK_COMPLETED = "rollback_completed"
}
/** The status of a service update. */
export interface ServiceUpdateStatus {
    State?: ServiceUpdateStatusStateEnum;
    StartedAt?: string;
    CompletedAt?: string;
    Message?: string;
}
/** The status of the service's tasks. Provided only when requested as part of a ServiceList operation. */
export interface ServiceServiceStatus {
    /** The number of tasks for the service currently in the Running state. */
    RunningTasks?: U64;
    /**
     * The number of tasks for the service desired to be running.
     * For replicated services, this is the replica count from the service spec.
     * For global services, this is computed by taking count of all tasks for the service with a Desired State other than Shutdown.
     */
    DesiredTasks?: U64;
    /**
     * The number of tasks for a job that are in the Completed state.
     * This field must be cross-referenced with the service type, as the value of 0 may mean the service is not in a job mode,
     * or it may mean the job-mode service has no tasks yet Completed.
     */
    CompletedTasks?: U64;
}
/** The status of the service when it is in one of ReplicatedJob or GlobalJob modes. Absent on Replicated and Global mode services. The JobIteration is an ObjectVersion, but unlike the Service's version, does not need to be sent with an update request. */
export interface ServiceJobStatus {
    /** JobIteration is a value increased each time a Job is executed, successfully or otherwise. \"Executed\", in this case, means the job as a whole has been started, not that an individual Task has been launched. A job is \"Executed\" when its ServiceSpec is updated. JobIteration can be used to disambiguate Tasks belonging to different executions of a job.  Though JobIteration will increase with each subsequent execution, it may not necessarily increase by 1, and so JobIteration should not be used to */
    JobIteration?: ObjectVersion;
    /** The last time, as observed by the server, that this job was started. */
    LastExecution?: string;
}
/** Swarm service details. */
export interface SwarmService {
    ID?: string;
    Version?: ObjectVersion;
    CreatedAt?: string;
    UpdatedAt?: string;
    Spec?: ServiceSpec;
    Endpoint?: ServiceEndpoint;
    UpdateStatus?: ServiceUpdateStatus;
    ServiceStatus?: ServiceServiceStatus;
    JobStatus?: ServiceJobStatus;
}
export type InspectStackSwarmServiceResponse = SwarmService;
export type InspectSwarmConfigResponse = SwarmConfig[];
export declare enum NodeSpecRoleEnum {
    EMPTY = "",
    WORKER = "worker",
    MANAGER = "manager"
}
export declare enum NodeSpecAvailabilityEnum {
    EMPTY = "",
    ACTIVE = "active",
    PAUSE = "pause",
    DRAIN = "drain"
}
export interface NodeSpec {
    /** Name for the node. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    /** Role of the node. */
    Role?: NodeSpecRoleEnum;
    /** Availability of the node. */
    Availability?: NodeSpecAvailabilityEnum;
}
export interface EngineDescriptionPlugins {
    Type?: string;
    Name?: string;
}
/** EngineDescription provides information about an engine. */
export interface EngineDescription {
    EngineVersion?: string;
    Labels?: Record<string, string>;
    Plugins?: EngineDescriptionPlugins[];
}
/** Information about the issuer of leaf TLS certificates and the trusted root CA certificate. */
export interface TlsInfo {
    /** The root CA certificate(s) that are used to validate leaf TLS certificates. */
    TrustRoot?: string;
    /** The base64-url-safe-encoded raw subject bytes of the issuer. */
    CertIssuerSubject?: string;
    /** The base64-url-safe-encoded raw public key bytes of the issuer. */
    CertIssuerPublicKey?: string;
}
export interface NodeDescription {
    Hostname?: string;
    Platform?: Platform;
    Resources?: ResourceObject;
    Engine?: EngineDescription;
    TLSInfo?: TlsInfo;
}
/** NodeState represents the state of a node. */
export declare enum NodeState {
    UNKNOWN = "unknown",
    DOWN = "down",
    READY = "ready",
    DISCONNECTED = "disconnected"
}
/** NodeStatus represents the status of a node.  It provides the current status of the node, as seen by the manager. */
export interface NodeStatus {
    State?: NodeState;
    Message?: string;
    /** IP address of the node. */
    Addr?: string;
}
/** Reachability represents the reachability of a node. */
export declare enum NodeReachability {
    UNKNOWN = "unknown",
    UNREACHABLE = "unreachable",
    REACHABLE = "reachable"
}
/** ManagerStatus represents the status of a manager.  It provides the current status of a node's manager component, if the node is a manager. */
export interface ManagerStatus {
    Leader?: boolean;
    Reachability?: NodeReachability;
    /** The IP address and port at which the manager is reachable. */
    Addr?: string;
}
/** Swarm node details. */
export interface SwarmNode {
    ID?: string;
    Version?: ObjectVersion;
    /** Date and time at which the node was added to the swarm in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    CreatedAt?: string;
    /** Date and time at which the node was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    UpdatedAt?: string;
    Spec?: NodeSpec;
    Description?: NodeDescription;
    Status?: NodeStatus;
    ManagerStatus?: ManagerStatus;
}
export type InspectSwarmNodeResponse = SwarmNode;
/** Orchestration configuration. */
export interface SwarmSpecOrchestration {
    /**
     * The number of historic tasks to keep per instance or node.
     * If negative, never remove completed or failed tasks.
     */
    TaskHistoryRetentionLimit?: I64;
}
/** Raft configuration. */
export interface SwarmSpecRaft {
    /** The number of log entries between snapshots. */
    SnapshotInterval?: U64;
    /** The number of snapshots to keep beyond the current snapshot. */
    KeepOldSnapshots?: U64;
    /** The number of log entries to keep around to sync up slow followers after a snapshot is created. */
    LogEntriesForSlowFollowers?: U64;
    /** The number of ticks that a follower will wait for a message from the leader before becoming a candidate and starting an election. `ElectionTick` must be greater than `HeartbeatTick`.  A tick currently defaults to one second, so these translate directly to seconds currently, but this is NOT guaranteed. */
    ElectionTick?: I64;
    /**
     * The number of ticks between heartbeats.
     * Every HeartbeatTick ticks, the leader will send a heartbeat to the followers.
     * A tick currently defaults to one second, so these translate directly to seconds currently, but this is NOT guaranteed.
     */
    HeartbeatTick?: I64;
}
/** Dispatcher configuration. */
export interface SwarmSpecDispatcher {
    /** The delay for an agent to send a heartbeat to the dispatcher. */
    HeartbeatPeriod?: I64;
}
export declare enum SwarmSpecCaConfigExternalCasProtocolEnum {
    EMPTY = "",
    CFSSL = "cfssl"
}
export interface SwarmSpecCaConfigExternalCas {
    /** Protocol for communication with the external CA (currently only `cfssl` is supported). */
    Protocol?: SwarmSpecCaConfigExternalCasProtocolEnum;
    /** URL where certificate signing requests should be sent. */
    URL?: string;
    /** An object with key/value pairs that are interpreted as protocol-specific options for the external CA driver. */
    Options?: Record<string, string>;
    /** The root CA certificate (in PEM format) this external CA uses to issue TLS certificates (assumed to be to the current swarm root CA certificate if not provided). */
    CACert?: string;
}
/** CA configuration. */
export interface SwarmSpecCaConfig {
    /** The duration node certificates are issued for. */
    NodeCertExpiry?: I64;
    /** Configuration for forwarding signing requests to an external certificate authority. */
    ExternalCAs?: SwarmSpecCaConfigExternalCas[];
    /** The desired signing CA certificate for all swarm node TLS leaf certificates, in PEM format. */
    SigningCACert?: string;
    /** The desired signing CA key for all swarm node TLS leaf certificates, in PEM format. */
    SigningCAKey?: string;
    /** An integer whose purpose is to force swarm to generate a new signing CA certificate and key, if none have been specified in `SigningCACert` and `SigningCAKey` */
    ForceRotate?: U64;
}
/** Parameters related to encryption-at-rest. */
export interface SwarmSpecEncryptionConfig {
    /** If set, generate a key and use it to lock data stored on the managers. */
    AutoLockManagers?: boolean;
}
/** The log driver to use for tasks created in the orchestrator if unspecified by a service.  Updating this value only affects new tasks. Existing tasks continue to use their previously configured log driver until recreated. */
export interface SwarmSpecTaskDefaultsLogDriver {
    /** The log driver to use as a default for new tasks. */
    Name?: string;
    /** Driver-specific options for the selected log driver, specified as key/value pairs. */
    Options?: Record<string, string>;
}
/** Defaults for creating tasks in this cluster. */
export interface SwarmSpecTaskDefaults {
    LogDriver?: SwarmSpecTaskDefaultsLogDriver;
}
/** User modifiable swarm configuration. */
export interface SwarmSpec {
    /** Name of the swarm. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    Orchestration?: SwarmSpecOrchestration;
    Raft?: SwarmSpecRaft;
    Dispatcher?: SwarmSpecDispatcher;
    CAConfig?: SwarmSpecCaConfig;
    EncryptionConfig?: SwarmSpecEncryptionConfig;
    TaskDefaults?: SwarmSpecTaskDefaults;
}
/** JoinTokens contains the tokens workers and managers need to join the swarm. */
export interface JoinTokens {
    /** The token workers can use to join the swarm. */
    Worker?: string;
    /** The token managers can use to join the swarm. */
    Manager?: string;
}
/** Docker-level information about the Swarm. */
export interface SwarmInspectInfo {
    /** The (Docker) ID of the swarm. */
    ID?: string;
    Version?: ObjectVersion;
    /** Date and time at which the swarm was initialised in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    CreatedAt?: string;
    /** Date and time at which the swarm was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    UpdatedAt?: string;
    Spec?: SwarmSpec;
    TLSInfo?: TlsInfo;
    /** Whether there is currently a root CA rotation in progress for the swarm */
    RootRotationInProgress?: boolean;
    /** DataPathPort specifies the data path port number for data traffic. Acceptable port range is 1024 to 49151. If no port is set or is set to 0, the default port (4789) is used. */
    DataPathPort?: number;
    /** Default Address Pool specifies default subnet pools for global scope networks. */
    DefaultAddrPool?: string[];
    /** SubnetSize specifies the subnet size of the networks created from the default subnet pool. */
    SubnetSize?: number;
    JoinTokens?: JoinTokens;
}
export type InspectSwarmResponse = SwarmInspectInfo;
/** Driver represents a driver (network, logging, secrets). */
export interface Driver {
    /** Name of the driver. */
    Name: string;
    /** Key/value map of driver-specific options. */
    Options?: Record<string, string>;
}
export interface SecretSpec {
    /** User-defined name of the secret. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    /**
     * Data is the data to store as a secret, formatted as a Base64-url-safe-encoded ([RFC 4648](https://tools.ietf.org/html/rfc4648#section-5)) string.
     * It must be empty if the Driver field is set, in which case the data is loaded from an external secret store.
     * The maximum allowed size is 500KB, as defined in [MaxSecretSize](https://pkg.go.dev/github.com/moby/swarmkit/v2@v2.0.0-20250103191802-8c1959736554/api/validation#MaxSecretSize).
     * This field is only used to _create_ a secret, and is not returned by other endpoints.
     */
    Data?: string;
    /** Name of the secrets driver used to fetch the secret's value from an external secret store. */
    Driver?: Driver;
    /**
     * Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template.
     * If no driver is set, no templating is used.
     */
    Templating?: Driver;
}
/** Swarm secret details. */
export interface SwarmSecret {
    ID?: string;
    Version?: ObjectVersion;
    CreatedAt?: string;
    UpdatedAt?: string;
    Spec?: SecretSpec;
}
export type InspectSwarmSecretResponse = SwarmSecret;
export type InspectSwarmServiceResponse = SwarmService;
export declare enum SwarmState {
    /** The Swarm is healthy, all nodes OK */
    Healthy = "Healthy",
    /** The Swarm is unhealthy */
    Unhealthy = "Unhealthy",
    /** Unknown case */
    Unknown = "Unknown"
}
/**
 * Swarm stack service list item.
 * Returned by `docker stack services --format json <NAME>`
 *
 * https://docs.docker.com/reference/cli/docker/stack/services/#format
 */
export interface SwarmStackServiceListItem {
    /** The *short* swarm service ID */
    ID?: string;
    /** The service name. */
    Name?: string;
    /** The service mode. */
    Mode?: string;
    /** The service replicas, formatted as string. */
    Replicas?: string;
    /** The image associated with service */
    Image?: string;
    /** Task exposed ports, formatted as a string. */
    Ports?: string;
}
/**
 * Swarm stack task list item.
 * Returned by `docker stack ps --format json <NAME>`
 *
 * https://docs.docker.com/reference/cli/docker/stack/ps/#format
 */
export interface SwarmStackTaskListItem {
    /** The task ID */
    ID?: string;
    /** Swarm stack task name. */
    Name?: string;
    /** The image associated with task */
    Image?: string;
    /** The node the task is running on */
    Node?: string;
    /** The task desired state. Matches 'CurrentState' when healthy. */
    DesiredState?: string;
    /** The task current state. Matches 'DesiredState' when healthy. */
    CurrentState?: string;
    /** An error message, if one exists */
    Error?: string;
    /** Task exposed ports, formatted as a string. */
    Ports?: string;
}
/**
 * All entities related to docker stack available over CLI.
 * Returned by:
 * ```
 * docker stack services --format json <STACK>
 * docker stack ps --format json <STACK>
 * ```
 */
export interface SwarmStack {
    /** Swarm stack name. */
    Name: string;
    /**
     * Swarm stack state.
     * - Healthy if all associated tasks match their desired state (or report no desired state)
     * - Unhealthy otherwise
     *
     * Not included in docker cli return, computed by Komodo
     */
    State: SwarmState;
    /** Services part of the stack */
    Services: SwarmStackServiceListItem[];
    /** Tasks part of the stack */
    Tasks: SwarmStackTaskListItem[];
}
export type InspectSwarmStackResponse = SwarmStack;
export declare enum TaskState {
    NEW = "new",
    ALLOCATED = "allocated",
    PENDING = "pending",
    ASSIGNED = "assigned",
    ACCEPTED = "accepted",
    PREPARING = "preparing",
    READY = "ready",
    STARTING = "starting",
    RUNNING = "running",
    COMPLETE = "complete",
    SHUTDOWN = "shutdown",
    FAILED = "failed",
    REJECTED = "rejected",
    REMOVE = "remove",
    ORPHANED = "orphaned"
}
/** represents the status of a container. */
export interface ContainerStatus {
    ContainerID?: string;
    PID?: I64;
    ExitCode?: I64;
}
/** represents the port status of a task's host ports whose service has published host ports */
export interface PortStatus {
    Ports?: EndpointPortConfig[];
}
/** represents the status of a task. */
export interface TaskStatus {
    Timestamp?: string;
    State?: TaskState;
    Message?: string;
    Err?: string;
    ContainerStatus?: ContainerStatus;
    PortStatus?: PortStatus;
}
/** Swarm task details. */
export interface SwarmTask {
    /** The ID of the task. */
    ID?: string;
    Version?: ObjectVersion;
    CreatedAt?: string;
    UpdatedAt?: string;
    /** Name of the task. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    Spec?: TaskSpec;
    /** The ID of the service this task is part of. */
    ServiceID?: string;
    Slot?: I64;
    /** The ID of the node that this task is on. */
    NodeID?: string;
    AssignedGenericResources?: GenericResources;
    Status?: TaskStatus;
    DesiredState?: TaskState;
    /** If the Service this Task belongs to is a job-mode service, contains the JobIteration of the Service this Task was created for. Absent if the Task was created for a Replicated or Global Service. */
    JobIteration?: ObjectVersion;
}
export type InspectSwarmTaskResponse = SwarmTask;
export type JsonObject = any;
export type JsonValue = any;
export type ListActionsResponse = ActionListItem[];
export type ListAlertersResponse = AlerterListItem[];
export declare enum PortTypeEnum {
    EMPTY = "",
    TCP = "tcp",
    UDP = "udp",
    SCTP = "sctp"
}
/** An open port on a container */
export interface Port {
    /** Host IP address that the container's port is mapped to */
    IP?: string;
    /** Port on the container */
    PrivatePort?: number;
    /** Port exposed on the host */
    PublicPort?: number;
    Type?: PortTypeEnum;
}
/** Container summary returned by container list apis. */
export interface ContainerListItem {
    /** The Server which hosts the container. */
    server_id?: string;
    /** The first name in Names, not including the initial '/' */
    name: string;
    /** The ID of this container */
    id?: string;
    /** The name of the image used when creating this container */
    image?: string;
    /** The ID of the image that this container was created from */
    image_id?: string;
    /** When the container was created */
    created?: I64;
    /** The size of files that have been created or changed by this container */
    size_rw?: I64;
    /** The total size of all the files in this container */
    size_root_fs?: I64;
    /** The state of this container (e.g. `exited`) */
    state: ContainerStateStatusEnum;
    /** Additional human-readable status of this container (e.g. `Exit 0`) */
    status?: string;
    /** The network mode */
    network_mode?: string;
    /** The network names attached to container */
    networks?: string[];
    /** Port mappings for the container */
    ports?: Port[];
    /** The volume names attached to container */
    volumes?: string[];
    /** The container stats, if they can be retreived. */
    stats?: ContainerStats;
    /**
     * The labels attached to container.
     * It's too big to send with container list,
     * can get it using InspectContainer
     */
    labels?: Record<string, string>;
}
export type ListAllDockerContainersResponse = ContainerListItem[];
/** An api key used to authenticate requests via request headers. */
export interface ApiKey {
    /** Unique key associated with secret */
    key: string;
    /** Hash of the secret */
    secret: string;
    /** User associated with the api key */
    user_id: string;
    /** Name associated with the api key for management */
    name: string;
    /** Timestamp of key creation */
    created_at: I64;
    /** Expiry of key, or 0 if never expires */
    expires: I64;
}
export type ListApiKeysForServiceUserResponse = ApiKey[];
export type ListApiKeysResponse = ApiKey[];
export interface BuildVersionResponseItem {
    version: Version;
    ts: I64;
}
export type ListBuildVersionsResponse = BuildVersionResponseItem[];
export type ListBuildersResponse = BuilderListItem[];
export type ListBuildsResponse = BuildListItem[];
export type ListCommonBuildExtraArgsResponse = string[];
export type ListCommonDeploymentExtraArgsResponse = string[];
export type ListCommonStackBuildExtraArgsResponse = string[];
export type ListCommonStackExtraArgsResponse = string[];
export interface ComposeProject {
    /** The compose project name. */
    name: string;
    /** The status of the project, as returned by docker. */
    status?: string;
    /** The compose files included in the project. */
    compose_files: string[];
}
export type ListComposeProjectsResponse = ComposeProject[];
export type ListDeploymentsResponse = DeploymentListItem[];
export type ListDockerContainersResponse = ContainerListItem[];
/** individual image layer information in response to ImageHistory operation */
export interface ImageHistoryResponseItem {
    Id: string;
    Created: I64;
    CreatedBy: string;
    Tags?: string[];
    Size: I64;
    Comment: string;
}
export type ListDockerImageHistoryResponse = ImageHistoryResponseItem[];
export interface ImageListItem {
    /** The first tag in `repo_tags`, or Id if no tags. */
    name: string;
    /** ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image. */
    id: string;
    /** ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry. */
    parent_id: string;
    /** Date and time at which the image was created as a Unix timestamp (number of seconds sinds EPOCH). */
    created: I64;
    /** Total size of the image including all layers it is composed of. */
    size: I64;
    /** Whether the image is in use by any container */
    in_use: boolean;
}
export type ListDockerImagesResponse = ImageListItem[];
export interface NetworkListItem {
    name?: string;
    id?: string;
    created?: string;
    scope?: string;
    driver?: string;
    enable_ipv6?: boolean;
    ipam_driver?: string;
    ipam_subnet?: string;
    ipam_gateway?: string;
    internal?: boolean;
    attachable?: boolean;
    ingress?: boolean;
    /** Whether the network is attached to one or more containers */
    in_use: boolean;
}
export type ListDockerNetworksResponse = NetworkListItem[];
export interface ProviderAccount {
    /** The account username. Required. */
    username: string;
    /** The account access token. Required. */
    token?: string;
}
export interface DockerRegistry {
    /** The docker provider domain. Default: `docker.io`. */
    domain: string;
    /** The accounts on the registry. Required. */
    accounts: ProviderAccount[];
    /**
     * Available organizations on the registry provider.
     * Used to push an image under an organization's repo rather than an account's repo.
     */
    organizations?: string[];
}
export type ListDockerRegistriesFromConfigResponse = DockerRegistry[];
export type ListDockerRegistryAccountsResponse = DockerRegistryAccount[];
export interface VolumeListItem {
    /** The name of the volume */
    name: string;
    driver: string;
    mountpoint: string;
    created?: string;
    scope: VolumeScopeEnum;
    /** Amount of disk space used by the volume (in bytes). This information is only available for volumes created with the `\"local\"` volume driver. For volumes created with other volume drivers, this field is set to `-1` (\"not available\") */
    size?: I64;
    /** Whether the volume is currently attached to any container */
    in_use: boolean;
}
export type ListDockerVolumesResponse = VolumeListItem[];
export type ListFullActionsResponse = Action[];
export type ListFullAlertersResponse = Alerter[];
export type ListFullBuildersResponse = Builder[];
export type ListFullBuildsResponse = Build[];
export type ListFullDeploymentsResponse = Deployment[];
export type ListFullProceduresResponse = Procedure[];
export type ListFullReposResponse = Repo[];
export type ListFullResourceSyncsResponse = ResourceSync[];
export type ListFullServersResponse = Server[];
export type ListFullStacksResponse = Stack[];
export type ListFullSwarmsResponse = Swarm[];
export type ListGitProviderAccountsResponse = GitProviderAccount[];
export interface GitProvider {
    /** The git provider domain. Default: `github.com`. */
    domain: string;
    /** Whether to use https. Default: true. */
    https: boolean;
    /** The accounts on the git provider. Required. */
    accounts: ProviderAccount[];
}
export type ListGitProvidersFromConfigResponse = GitProvider[];
export type ListOnboardingKeysResponse = OnboardingKey[];
export type UserTarget = 
/** User Id */
{
    type: "User";
    id: string;
}
/** UserGroup Id */
 | {
    type: "UserGroup";
    id: string;
};
/** Representation of a User or UserGroups permission on a resource. */
export interface Permission {
    /** The id of the permission document */
    _id?: MongoId;
    /** The target User / UserGroup */
    user_target: UserTarget;
    /** The target resource */
    resource_target: ResourceTarget;
    /** The permission level for the [user_target] on the [resource_target]. */
    level?: PermissionLevel;
    /** Any specific permissions for the [user_target] on the [resource_target]. */
    specific?: Array<SpecificPermission>;
}
export type ListPermissionsResponse = Permission[];
export declare enum ProcedureState {
    /** Currently running */
    Running = "Running",
    /** Last run successful */
    Ok = "Ok",
    /** Last run failed */
    Failed = "Failed",
    /** Other case (never run) */
    Unknown = "Unknown"
}
export interface ProcedureListItemInfo {
    /** Number of stages procedure has. */
    stages: I64;
    /** Reflect whether last run successful / currently running. */
    state: ProcedureState;
    /** Procedure last successful run timestamp in ms. */
    last_run_at?: I64;
    /**
     * If the procedure has schedule enabled, this is the
     * next scheduled run time in unix ms.
     */
    next_scheduled_run?: I64;
    /**
     * If there is an error parsing schedule expression,
     * it will be given here.
     */
    schedule_error?: string;
}
export type ProcedureListItem = ResourceListItem<ProcedureListItemInfo>;
export type ListProceduresResponse = ProcedureListItem[];
export declare enum RepoState {
    /** Unknown case */
    Unknown = "Unknown",
    /** Last clone / pull successful (or never cloned) */
    Ok = "Ok",
    /** Last clone / pull failed */
    Failed = "Failed",
    /** Currently cloning */
    Cloning = "Cloning",
    /** Currently pulling */
    Pulling = "Pulling",
    /** Currently building */
    Building = "Building"
}
export interface RepoListItemInfo {
    /** The server that repo sits on. */
    server_id: string;
    /** The builder that builds the repo. */
    builder_id: string;
    /** Repo last cloned / pulled timestamp in ms. */
    last_pulled_at: I64;
    /** Repo last built timestamp in ms. */
    last_built_at: I64;
    /** The git provider domain */
    git_provider: string;
    /** The configured repo */
    repo: string;
    /** The configured branch */
    branch: string;
    /** Full link to the repo. */
    repo_link: string;
    /** The repo state */
    state: RepoState;
    /** If the repo is cloned, will be the cloned short commit hash. */
    cloned_hash?: string;
    /** If the repo is cloned, will be the cloned commit message. */
    cloned_message?: string;
    /** If the repo is built, will be the latest built short commit hash. */
    built_hash?: string;
    /** Will be the latest remote short commit hash. */
    latest_hash?: string;
}
export type RepoListItem = ResourceListItem<RepoListItemInfo>;
export type ListReposResponse = RepoListItem[];
export declare enum ResourceSyncState {
    /** Currently syncing */
    Syncing = "Syncing",
    /** Updates pending */
    Pending = "Pending",
    /** Last sync successful (or never synced). No Changes pending */
    Ok = "Ok",
    /** Last sync failed */
    Failed = "Failed",
    /** Other case */
    Unknown = "Unknown"
}
export interface ResourceSyncListItemInfo {
    /** Unix timestamp of last sync, or 0 */
    last_sync_ts: I64;
    /** Whether sync is `files_on_host` mode. */
    files_on_host: boolean;
    /** Whether sync has file contents defined. */
    file_contents: boolean;
    /** Whether sync has `managed` mode enabled. */
    managed: boolean;
    /** Resource paths to the files. */
    resource_path: string[];
    /** Linked repo, if one is attached. */
    linked_repo: string;
    /** The git provider domain. */
    git_provider: string;
    /** The Github repo used as the source of the sync resources */
    repo: string;
    /** The branch of the repo */
    branch: string;
    /** Full link to the repo. */
    repo_link: string;
    /** Short commit hash of last sync, or empty string */
    last_sync_hash?: string;
    /** Commit message of last sync, or empty string */
    last_sync_message?: string;
    /** State of the sync. Reflects whether most recent sync successful. */
    state: ResourceSyncState;
}
export type ResourceSyncListItem = ResourceListItem<ResourceSyncListItemInfo>;
export type ListResourceSyncsResponse = ResourceSyncListItem[];
/** A scheduled Action / Procedure run. */
export interface Schedule {
    /** Procedure or Alerter */
    target: ResourceTarget;
    /** Readable name of the target resource */
    name: string;
    /** The format of the schedule expression */
    schedule_format: ScheduleFormat;
    /** The schedule for the run */
    schedule: string;
    /** Whether the scheduled run is enabled */
    enabled: boolean;
    /** Custom schedule timezone if it exists */
    schedule_timezone: string;
    /** Last run timestamp in ms. */
    last_run_at?: I64;
    /** Next scheduled run time in unix ms. */
    next_scheduled_run?: I64;
    /**
     * If there is an error parsing schedule expression,
     * it will be given here.
     */
    schedule_error?: string;
    /** Resource tags. */
    tags: string[];
}
export type ListSchedulesResponse = Schedule[];
export type ListSecretsResponse = string[];
export declare enum ServerState {
    /** Server health check passing. */
    Ok = "Ok",
    /** Server is unreachable. */
    NotOk = "NotOk",
    /** Server is disabled. */
    Disabled = "Disabled"
}
export interface ServerListItemInfo {
    /** The server's state. */
    state: ServerState;
    /** Region of the server. */
    region: string;
    /** Address of the server, or null if empty. */
    address?: string;
    /**
     * External address of the server (reachable by users).
     * Used with links.
     */
    external_address?: string;
    /** Host public ip, if it could be resolved. */
    public_ip?: string;
    /** Whether server is configured to send disconnected alerts. */
    send_unreachable_alerts: boolean;
    /** Whether server is configured to send cpu alerts. */
    send_cpu_alerts: boolean;
    /** Whether server is configured to send mem alerts. */
    send_mem_alerts: boolean;
    /** Whether server is configured to send disk alerts. */
    send_disk_alerts: boolean;
    /** Whether server is configured to send version mismatch alerts. */
    send_version_mismatch_alerts: boolean;
    /** The Komodo Periphery version. */
    version?: string;
    /** The public key of Periphery */
    public_key?: string;
    /**
     * If a Periphery fails to authenticate to Core with invalid Periphery public key,
     * it will be stored here to accept the connection later on.
     */
    attempted_public_key?: string;
    /**
     * Whether server is configured to send unreachable alerts.
     * Whether terminals are disabled for this Server.
     */
    terminals_disabled: boolean;
    /** Whether container terminals are disabled for this Server. */
    container_terminals_disabled: boolean;
}
export type ServerListItem = ResourceListItem<ServerListItemInfo>;
export type ListServersResponse = ServerListItem[];
/** Swarm service list item. */
export interface SwarmServiceListItem {
    ID?: string;
    /** Name of the service. */
    Name?: string;
    /** The image associated with service */
    Image?: string;
    /** Runtime is the type of runtime specified for the task executor. */
    Runtime?: string;
    /** Condition for restart. */
    Restart?: TaskSpecRestartPolicyConditionEnum;
    /** Number of replicas */
    Replicas?: I64;
    /**
     * Swarm service state.
     * - Healthy if all associated tasks match their desired state (or report no desired state)
     * - Unhealthy otherwise
     *
     * Not included in docker cli return, computed by Komodo
     */
    State: SwarmState;
    CreatedAt?: string;
    UpdatedAt?: string;
}
export interface StackService {
    /** The service name */
    service: string;
    /** The service image */
    image: string;
    /** The container (Server mode) */
    container?: ContainerListItem;
    /** The service (Swarm mode) */
    swarm_service?: SwarmServiceListItem;
    /** Whether there is an update available for this services image. */
    update_available: boolean;
}
export type ListStackServicesResponse = StackService[];
export declare enum StackState {
    /** The stack is currently re/deploying */
    Deploying = "deploying",
    /** All containers are running. */
    Running = "running",
    /** All containers are paused */
    Paused = "paused",
    /** All contianers are stopped */
    Stopped = "stopped",
    /** All containers are created */
    Created = "created",
    /** All containers are restarting */
    Restarting = "restarting",
    /** All containers are dead */
    Dead = "dead",
    /** All containers are removing */
    Removing = "removing",
    /** The containers are in a mix of states */
    Unhealthy = "unhealthy",
    /** The stack is not deployed */
    Down = "down",
    /** Server not reachable for status */
    Unknown = "unknown"
}
export interface StackServiceWithUpdate {
    service: string;
    /** The service's image */
    image: string;
    /** Whether there is a newer image available for this service */
    update_available: boolean;
}
export interface StackListItemInfo {
    /** The swarm that stack is deployed on, when in Swarm mode. */
    swarm_id: string;
    /** The server that stack is deployed on, when in Server mode. */
    server_id: string;
    /** Whether stack is using files on host mode */
    files_on_host: boolean;
    /** Whether stack has file contents defined. */
    file_contents: boolean;
    /** Linked repo, if one is attached. */
    linked_repo: string;
    /** The git provider domain */
    git_provider: string;
    /** The configured repo */
    repo: string;
    /** The configured branch */
    branch: string;
    /** Full link to the repo. */
    repo_link: string;
    /** The stack state */
    state: StackState;
    /** A string given by docker conveying the status of the stack. */
    status?: string;
    /**
     * The services that are part of the stack.
     * If deployed, will be `deployed_services`.
     * Otherwise, its `latest_services`
     */
    services: StackServiceWithUpdate[];
    /**
     * Whether the compose project is missing on the host.
     * Ie, it does not show up in `docker compose ls`.
     * If true, and the stack is not Down, this is an unhealthy state.
     */
    project_missing: boolean;
    /**
     * If any compose files are missing in the repo, the path will be here.
     * If there are paths here, this is an unhealthy state, and deploying will fail.
     */
    missing_files: string[];
    /** Deployed short commit hash, or null. Only for repo based stacks. */
    deployed_hash?: string;
    /** Latest short commit hash, or null. Only for repo based stacks */
    latest_hash?: string;
}
export type StackListItem = ResourceListItem<StackListItemInfo>;
export type ListStacksResponse = StackListItem[];
/**
 * Swarm config list item.
 * Returned by `docker config ls --format json`
 */
export interface SwarmConfigListItem {
    /** User-defined name of the config. */
    Name?: string;
    ID?: string;
    CreatedAt?: string;
    UpdatedAt?: string;
    /**
     * User-defined key/value metadata, formatted as a string:
     * `"lab1=val1,lab2=val2"`.
     */
    Labels?: string;
}
export type ListSwarmConfigsResponse = SwarmConfigListItem[];
/** Swarm node list item. */
export interface SwarmNodeListItem {
    ID?: string;
    /** Name for the node. */
    Name?: string;
    Hostname?: string;
    /** Role of the node. */
    Role?: NodeSpecRoleEnum;
    /** Availability of the node. */
    Availability?: NodeSpecAvailabilityEnum;
    /** State of the node */
    State?: NodeState;
    /** Date and time at which the node was added to the swarm in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    CreatedAt?: string;
    /** Date and time at which the node was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds. */
    UpdatedAt?: string;
}
export type ListSwarmNodesResponse = SwarmNodeListItem[];
/** Swarm secret list item. */
export interface SwarmSecretListItem {
    ID?: string;
    /** User-defined name of the secret. */
    Name?: string;
    /** Name of the secrets driver used to fetch the secret's value from an external secret store. */
    Driver?: string;
    /**
     * Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template.
     * If no driver is set, no templating is used.
     */
    Templating?: string;
    CreatedAt?: string;
    UpdatedAt?: string;
}
export type ListSwarmSecretsResponse = SwarmSecretListItem[];
export type ListSwarmServicesResponse = SwarmServiceListItem[];
/**
 * Swarm stack list item.
 * Returned by `docker stack ls --format json`
 *
 * https://docs.docker.com/reference/cli/docker/stack/ls/#format
 */
export interface SwarmStackListItem {
    /** Swarm stack name. */
    Name?: string;
    /**
     * Swarm stack state.
     * - Healthy if all associated tasks match their desired state
     * - Unhealthy otherwise
     *
     * Not included in docker cli return, computed by Komodo
     */
    State?: SwarmState;
    /** Number of services which are part of the stack */
    Services?: string;
    /** The stack orchestrator */
    Orchestrator?: string;
    /** The stack namespace */
    Namespace?: string;
}
export type ListSwarmStacksResponse = SwarmStackListItem[];
/** Swarm task list item. */
export interface SwarmTaskListItem {
    /** The ID of the task. */
    ID?: string;
    /** Name of the task. */
    Name?: string;
    /** The ID of the node that this task is on. */
    NodeID?: string;
    /** The ID of the service this task is part of. */
    ServiceID?: string;
    /** The ID of container associated with this task. */
    ContainerID?: string;
    State?: TaskState;
    DesiredState?: TaskState;
    CreatedAt?: string;
    UpdatedAt?: string;
}
export type ListSwarmTasksResponse = SwarmTaskListItem[];
export interface SwarmListItemInfo {
    /** Servers part of the swarm */
    server_ids: string[];
    /** The Swarm state */
    state: SwarmState;
    /**
     * If there is an error reaching
     * Swarm, message will be given here.
     */
    err?: string;
}
export type SwarmListItem = ResourceListItem<SwarmListItemInfo>;
export type ListSwarmsResponse = SwarmListItem[];
/** Information about a process on the system. */
export interface SystemProcess {
    /** The process PID */
    pid: number;
    /** The process name */
    name: string;
    /** The path to the process executable */
    exe?: string;
    /** The command used to start the process */
    cmd: string[];
    /** The time the process was started */
    start_time?: number;
    /**
     * The cpu usage percentage of the process.
     * This is in core-percentage, eg 100% is 1 full core, and
     * an 8 core machine would max at 800%.
     */
    cpu_perc: number;
    /** The memory usage of the process in MB */
    mem_mb: number;
    /** Process disk read in KB/s */
    disk_read_kb: number;
    /** Process disk write in KB/s */
    disk_write_kb: number;
}
export type ListSystemProcessesResponse = SystemProcess[];
export type ListTagsResponse = Tag[];
export type TerminalTarget = {
    type: "Server";
    params: {
        server?: string;
    };
} | {
    type: "Container";
    params: {
        server: string;
        container: string;
    };
} | {
    type: "Stack";
    params: {
        stack: string;
        service?: string;
    };
} | {
    type: "Deployment";
    params: {
        deployment: string;
    };
};
/**
 * Represents an active terminal on a server.
 * Retrieve with [ListTerminals][crate::api::read::server::ListTerminals].
 */
export interface Terminal {
    /** The name of the terminal. */
    name: string;
    /** The target resource of the Terminal. */
    target: TerminalTarget;
    /** The command used to init the shell. */
    command: string;
    /** The size of the terminal history in memory. */
    stored_size_kb: number;
    /**
     * When the Terminal was created.
     * Unix timestamp milliseconds.
     */
    created_at: I64;
}
export type ListTerminalsResponse = Terminal[];
export type ListUserGroupsResponse = UserGroup[];
export type ListUserTargetPermissionsResponse = Permission[];
export type ListUsersResponse = User[];
export type ListVariablesResponse = Variable[];
/** The response for [LoginLocalUser] */
export type LoginLocalUserResponse = JwtResponse;
export type MongoDocument = any;
export interface ProcedureQuerySpecifics {
}
export type ProcedureQuery = ResourceQuery<ProcedureQuerySpecifics>;
export type PushRecentlyViewedResponse = NoData;
export interface RepoQuerySpecifics {
    /** Filter repos by their repo. */
    repos: string[];
}
export type RepoQuery = ResourceQuery<RepoQuerySpecifics>;
export interface ResourceSyncQuerySpecifics {
    /** Filter syncs by their repo. */
    repos: string[];
}
export type ResourceSyncQuery = ResourceQuery<ResourceSyncQuerySpecifics>;
export type SearchContainerLogResponse = Log;
export type SearchDeploymentLogResponse = Log;
export type SearchStackLogResponse = Log;
export type SearchSwarmServiceLogResponse = Log;
export interface ServerQuerySpecifics {
}
/** Server-specific query */
export type ServerQuery = ResourceQuery<ServerQuerySpecifics>;
export type SetLastSeenUpdateResponse = NoData;
/** Response for [SignUpLocalUser]. */
export type SignUpLocalUserResponse = JwtResponse;
export interface StackQuerySpecifics {
    /**
     * Query only for Stacks on these Servers.
     * If empty, does not filter by Server.
     * Only accepts Server id (not name).
     */
    server_ids?: string[];
    /**
     * Query only for Stacks with these linked repos.
     * Only accepts Repo id (not name).
     */
    linked_repos?: string[];
    /** Filter syncs by their repo. */
    repos?: string[];
    /** Query only for Stack with available image updates. */
    update_available?: boolean;
}
export type StackQuery = ResourceQuery<StackQuerySpecifics>;
export interface SwarmQuerySpecifics {
    /** Filter swarms by server ids. */
    servers: string[];
}
export type SwarmQuery = ResourceQuery<SwarmQuerySpecifics>;
export type UpdateDockerRegistryAccountResponse = DockerRegistryAccount;
export type UpdateGitProviderAccountResponse = GitProviderAccount;
export type UpdateOnboardingKeyResponse = OnboardingKey;
export type UpdatePermissionOnResourceTypeResponse = NoData;
export type UpdatePermissionOnTargetResponse = NoData;
export type UpdateProcedureResponse = Procedure;
export type UpdateResourceMetaResponse = NoData;
export type UpdateServiceUserDescriptionResponse = User;
export type UpdateUserAdminResponse = NoData;
export type UpdateUserBasePermissionsResponse = NoData;
export type UpdateUserPasswordResponse = NoData;
export type UpdateUserUsernameResponse = NoData;
export type UpdateVariableDescriptionResponse = Variable;
export type UpdateVariableIsSecretResponse = Variable;
export type UpdateVariableValueResponse = Variable;
export type _PartialActionConfig = Partial<ActionConfig>;
export type _PartialAlerterConfig = Partial<AlerterConfig>;
export type _PartialAwsBuilderConfig = Partial<AwsBuilderConfig>;
export type _PartialBuildConfig = Partial<BuildConfig>;
export type _PartialBuilderConfig = Partial<BuilderConfig>;
export type _PartialDeploymentConfig = Partial<DeploymentConfig>;
export type _PartialDockerRegistryAccount = Partial<DockerRegistryAccount>;
export type _PartialGitProviderAccount = Partial<GitProviderAccount>;
export type _PartialProcedureConfig = Partial<ProcedureConfig>;
export type _PartialRepoConfig = Partial<RepoConfig>;
export type _PartialResourceSyncConfig = Partial<ResourceSyncConfig>;
export type _PartialServerBuilderConfig = Partial<ServerBuilderConfig>;
export type _PartialServerConfig = Partial<ServerConfig>;
export type _PartialStackConfig = Partial<StackConfig>;
export type _PartialSwarmConfig = Partial<SwarmConfig>;
export type _PartialTag = Partial<Tag>;
export type _PartialUrlBuilderConfig = Partial<UrlBuilderConfig>;
export interface __Serror {
    error: string;
    trace: string[];
}
export type _Serror = __Serror;
/** **Admin only.** Add a user to a user group. Response: [UserGroup] */
export interface AddUserToUserGroup {
    /** The name or id of UserGroup that user should be added to. */
    user_group: string;
    /** The id or username of the user to add */
    user: string;
}
/** Configuration for an AWS builder. */
export interface AwsBuilderConfig {
    /** The AWS region to create the instance in */
    region: string;
    /** The instance type to create for the build */
    instance_type: string;
    /** The size of the builder volume in gb */
    volume_gb: number;
    /**
     * The EC2 ami id to create.
     * The ami should have the periphery client configured to start on startup,
     * and should have the necessary github / dockerhub accounts configured.
     */
    ami_id?: string;
    /** The subnet id to create the instance in. */
    subnet_id?: string;
    /** The key pair name to attach to the instance */
    key_pair_name?: string;
    /**
     * Whether to assign the instance a public IP address.
     * Likely needed for the instance to be able to reach the open internet.
     */
    assign_public_ip?: boolean;
    /**
     * Whether core should use the public IP address to communicate with periphery on the builder.
     * If false, core will communicate with the instance using the private IP.
     */
    use_public_ip?: boolean;
    /**
     * The security group ids to attach to the instance.
     * This should include a security group to allow core inbound access to the periphery port.
     */
    security_group_ids?: string[];
    /** The user data to deploy the instance with. */
    user_data?: string;
    /**
     * The port periphery will be running on.
     * Default: `8120`
     */
    port: number;
    use_https: boolean;
    /**
     * An expected public key associated with Periphery private key.
     * If empty, doesn't validate Periphery public key.
     */
    periphery_public_key?: string;
    /** Whether to validate the Periphery tls certificates. */
    insecure_tls: boolean;
    /** Which git providers are available on the AMI */
    git_providers?: GitProvider[];
    /** Which docker registries are available on the AMI. */
    docker_registries?: DockerRegistry[];
    /** Which secrets are available on the AMI. */
    secrets?: string[];
}
/**
 * **Admin only.** Backs up the Komodo Core database to compressed jsonl files.
 * Response: [Update]. Aliases: `backup-database`, `backup-db`, `backup`.
 *
 * Mount a folder to `/backups`, and Core will use it to create
 * timestamped database dumps, which can be restored using
 * the Komodo CLI.
 *
 * https://komo.do/docs/setup/backup
 */
export interface BackupCoreDatabase {
}
/** Builds multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchBuildRepo {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* repos
     * foo-*
     * # add some more
     * extra-repo-1, extra-repo-2
     * ```
     */
    pattern: string;
}
/** Clones multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchCloneRepo {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* repos
     * foo-*
     * # add some more
     * extra-repo-1, extra-repo-2
     * ```
     */
    pattern: string;
}
/**
 * Delete all terminals on many or all Servers.
 * Response: [NoData]
 */
export interface BatchDeleteAllTerminals {
    /** Optional structured query to filter servers. */
    query?: ServerQuery;
}
/** Deploys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchDeploy {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* deployments
     * foo-*
     * # add some more
     * extra-deployment-1, extra-deployment-2
     * ```
     */
    pattern: string;
}
/** Deploys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchDeployStack {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* stacks
     * foo-*
     * # add some more
     * extra-stack-1, extra-stack-2
     * ```
     */
    pattern: string;
}
/** Deploys multiple Stacks if changed in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchDeployStackIfChanged {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* stacks
     * foo-*
     * # add some more
     * extra-stack-1, extra-stack-2
     * ```
     */
    pattern: string;
}
/** Destroys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchDestroyDeployment {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* deployments
     * foo-*
     * # add some more
     * extra-deployment-1, extra-deployment-2
     * ```
     */
    pattern: string;
}
/** Destroys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchDestroyStack {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     * d
     * Example:
     * ```text
     * # match all foo-* stacks
     * foo-*
     * # add some more
     * extra-stack-1, extra-stack-2
     * ```
     */
    pattern: string;
}
export interface BatchExecutionResponseItemErr {
    name: string;
    error: _Serror;
}
/** Pulls multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchPullRepo {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* repos
     * foo-*
     * # add some more
     * extra-repo-1, extra-repo-2
     * ```
     */
    pattern: string;
}
/** Pulls multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchPullStack {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* stacks
     * foo-*
     * # add some more
     * extra-stack-1, extra-stack-2
     * ```
     */
    pattern: string;
}
/** Runs multiple Actions in parallel that match pattern. Response: [BatchExecutionResponse] */
export interface BatchRunAction {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* actions
     * foo-*
     * # add some more
     * extra-action-1, extra-action-2
     * ```
     */
    pattern: string;
}
/** Runs multiple builds in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchRunBuild {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* builds
     * foo-*
     * # add some more
     * extra-build-1, extra-build-2
     * ```
     */
    pattern: string;
}
/** Runs multiple Procedures in parallel that match pattern. Response: [BatchExecutionResponse]. */
export interface BatchRunProcedure {
    /**
     * Id or name or wildcard pattern or regex.
     * Supports multiline and comma delineated combinations of the above.
     *
     * Example:
     * ```text
     * # match all foo-* procedures
     * foo-*
     * # add some more
     * extra-procedure-1, extra-procedure-2
     * ```
     */
    pattern: string;
}
/**
 * Builds the target repo, using the attached builder. Response: [Update].
 *
 * Note. Repo must have builder attached at `builder_id`.
 *
 * 1. Spawns the target builder instance (For AWS type. For Server type, just use CloneRepo).
 * 2. Clones the repo on the builder using `git clone https://{$token?}@github.com/${repo} -b ${branch}`.
 * The token will only be used if a github account is specified,
 * and must be declared in the periphery configuration on the builder instance.
 * 3. If `on_clone` and `on_pull` are specified, they will be executed.
 * `on_clone` will be executed before `on_pull`.
 */
export interface BuildRepo {
    /** Id or name */
    repo: string;
}
/** Item in [GetBuildMonthlyStatsResponse] */
export interface BuildStatsDay {
    time: number;
    count: number;
    ts: number;
}
/**
 * Cancels the target build.
 * Only does anything if the build is `building` when called.
 * Response: [Update]
 */
export interface CancelBuild {
    /** Can be id or name */
    build: string;
}
/**
 * Cancels the target repo build.
 * Only does anything if the repo build is `building` when called.
 * Response: [Update]
 */
export interface CancelRepoBuild {
    /** Can be id or name */
    repo: string;
}
/**
 * **Admin only.** Clears all repos from the Core repo cache.
 * Response: [Update]
 */
export interface ClearRepoCache {
}
/**
 * Clones the target repo. Response: [Update].
 *
 * Note. Repo must have server attached at `server_id`.
 *
 * 1. Clones the repo on the target server using `git clone https://{$token?}@github.com/${repo} -b ${branch}`.
 * The token will only be used if a github account is specified,
 * and must be declared in the periphery configuration on the target server.
 * 2. If `on_clone` and `on_pull` are specified, they will be executed.
 * `on_clone` will be executed before `on_pull`.
 */
export interface CloneRepo {
    /** Id or name */
    repo: string;
}
/**
 * **Admin only.** Close the Alert at the given id.
 * Response: [NoData]
 */
export interface CloseAlert {
    /** The id of the Alert to close. */
    id: string;
}
/**
 * Exports matching resources, and writes to the target sync's resource file. Response: [Update]
 *
 * Note. Will fail if the Sync is not `managed`.
 */
export interface CommitSync {
    /** Id or name */
    sync: string;
}
export interface ConfigSpec {
    /** User-defined name of the config. */
    Name?: string;
    /** User-defined key/value metadata. */
    Labels?: Record<string, string>;
    /**
     * Data is the data to store as a config, formatted as a Base64-url-safe-encoded ([RFC 4648](https://tools.ietf.org/html/rfc4648#section-5)) string.
     * It must be empty if the Driver field is set, in which case the data is loaded from an external secret store.
     * The maximum allowed size is 500KB, as defined in [MaxSecretSize](https://pkg.go.dev/github.com/moby/swarmkit/v2@v2.0.0-20250103191802-8c1959736554/api/validation#MaxSecretSize).
     */
    Data?: string;
    /** Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template. If no driver is set, no templating is used. */
    Templating?: Driver;
}
/**
 * Configures the behavior of [CreateTerminal] if the
 * specified terminal name already exists.
 */
export declare enum TerminalRecreateMode {
    /**
     * Never kill the old terminal if it already exists.
     * If the init command is different, returns error.
     */
    Never = "Never",
    /** Always kill the old terminal and create new one */
    Always = "Always",
    /** Only kill and recreate if the command is different. */
    DifferentCommand = "DifferentCommand"
}
/** Specify the container terminal mode (exec or attach) */
export declare enum ContainerTerminalMode {
    Exec = "exec",
    Attach = "attach"
}
/** Args to init the Terminal if needed. */
export interface InitTerminal {
    /**
     * The shell command (eg `bash`) to init the shell.
     *
     * Default:
     * - Server: Configured on each Periphery
     * - Container: `sh`
     */
    command?: string;
    /** Default: `Never` */
    recreate?: TerminalRecreateMode;
    /**
     * Only relevant for container-type terminals.
     * Specify the container terminal mode (`exec` or `attach`).
     * Default: `exec`
     */
    mode?: ContainerTerminalMode;
}
/** Connect to a Terminal. */
export interface ConnectTerminalQuery {
    /** The target to create terminal for. */
    target: TerminalTarget;
    /**
     * Terminal name to connect to.
     * If it may not exist yet, also pass 'init' params
     * to include initialization.
     * Default: Depends on target.
     */
    terminal?: string;
    /**
     * Pass to init the terminal session
     * for when the terminal doesn't already exist.
     *
     * Example: ?...(query)&init[command]=bash&init[recreate]=DifferentCommand
     */
    init?: InitTerminal;
}
/** Blkio stats entry.  This type is Linux-specific and omitted for Windows containers. */
export interface ContainerBlkioStatEntry {
    major?: U64;
    minor?: U64;
    op?: string;
    value?: U64;
}
/**
 * BlkioStats stores all IO service stats for data read and write.
 * This type is Linux-specific and holds many fields that are specific to cgroups v1.
 * On a cgroup v2 host, all fields other than `io_service_bytes_recursive` are omitted or `null`.
 * This type is only populated on Linux and omitted for Windows containers.
 */
export interface ContainerBlkioStats {
    io_service_bytes_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_serviced_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_queue_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_service_time_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_wait_time_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_merged_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    io_time_recursive?: ContainerBlkioStatEntry[];
    /**
     * This field is only available when using Linux containers with cgroups v1.
     * It is omitted or `null` when using cgroups v2.
     */
    sectors_recursive?: ContainerBlkioStatEntry[];
}
/** All CPU stats aggregated since container inception. */
export interface ContainerCpuUsage {
    /** Total CPU time consumed in nanoseconds (Linux) or 100's of nanoseconds (Windows). */
    total_usage?: U64;
    /**
     * Total CPU time (in nanoseconds) consumed per core (Linux).
     * This field is Linux-specific when using cgroups v1.
     * It is omitted when using cgroups v2 and Windows containers.
     */
    percpu_usage?: U64[];
    /**
     * Time (in nanoseconds) spent by tasks of the cgroup in kernel mode (Linux),
     * or time spent (in 100's of nanoseconds) by all container processes in kernel mode (Windows).
     * Not populated for Windows containers using Hyper-V isolation.
     */
    usage_in_kernelmode?: U64;
    /**
     * Time (in nanoseconds) spent by tasks of the cgroup in user mode (Linux),
     * or time spent (in 100's of nanoseconds) by all container processes in kernel mode (Windows).
     * Not populated for Windows containers using Hyper-V isolation.
     */
    usage_in_usermode?: U64;
}
/**
 * CPU throttling stats of the container.
 * This type is Linux-specific and omitted for Windows containers.
 */
export interface ContainerThrottlingData {
    /** Number of periods with throttling active. */
    periods?: U64;
    /** Number of periods when the container hit its throttling limit. */
    throttled_periods?: U64;
    /** Aggregated time (in nanoseconds) the container was throttled for. */
    throttled_time?: U64;
}
/** CPU related info of the container */
export interface ContainerCpuStats {
    /** All CPU stats aggregated since container inception. */
    cpu_usage?: ContainerCpuUsage;
    /**
     * System Usage.
     * This field is Linux-specific and omitted for Windows containers.
     */
    system_cpu_usage?: U64;
    /**
     * Number of online CPUs.
     * This field is Linux-specific and omitted for Windows containers.
     */
    online_cpus?: number;
    /**
     * CPU throttling stats of the container.
     * This type is Linux-specific and omitted for Windows containers.
     */
    throttling_data?: ContainerThrottlingData;
}
/**
 * Aggregates all memory stats since container inception on Linux.
 * Windows returns stats for commit and private working set only.
 */
export interface ContainerMemoryStats {
    /**
     * Current `res_counter` usage for memory.
     * This field is Linux-specific and omitted for Windows containers.
     */
    usage?: U64;
    /**
     * Maximum usage ever recorded.
     * This field is Linux-specific and only supported on cgroups v1.
     * It is omitted when using cgroups v2 and for Windows containers.
     */
    max_usage?: U64;
    /**
     * All the stats exported via memory.stat. when using cgroups v2.
     * This field is Linux-specific and omitted for Windows containers.
     */
    stats?: Record<string, U64>;
    /** Number of times memory usage hits limits.  This field is Linux-specific and only supported on cgroups v1. It is omitted when using cgroups v2 and for Windows containers. */
    failcnt?: U64;
    /** This field is Linux-specific and omitted for Windows containers. */
    limit?: U64;
    /**
     * Committed bytes.
     * This field is Windows-specific and omitted for Linux containers.
     */
    commitbytes?: U64;
    /**
     * Peak committed bytes.
     * This field is Windows-specific and omitted for Linux containers.
     */
    commitpeakbytes?: U64;
    /**
     * Private working set.
     * This field is Windows-specific and omitted for Linux containers.
     */
    privateworkingset?: U64;
}
/** Aggregates the network stats of one container */
export interface ContainerNetworkStats {
    /** Bytes received. Windows and Linux. */
    rx_bytes?: U64;
    /** Packets received. Windows and Linux. */
    rx_packets?: U64;
    /**
     * Received errors. Not used on Windows.
     * This field is Linux-specific and always zero for Windows containers.
     */
    rx_errors?: U64;
    /** Incoming packets dropped. Windows and Linux. */
    rx_dropped?: U64;
    /** Bytes sent. Windows and Linux. */
    tx_bytes?: U64;
    /** Packets sent. Windows and Linux. */
    tx_packets?: U64;
    /**
     * Sent errors. Not used on Windows.
     * This field is Linux-specific and always zero for Windows containers.
     */
    tx_errors?: U64;
    /** Outgoing packets dropped. Windows and Linux. */
    tx_dropped?: U64;
    /**
     * Endpoint ID. Not used on Linux.
     * This field is Windows-specific and omitted for Linux containers.
     */
    endpoint_id?: string;
    /**
     * Instance ID. Not used on Linux.
     * This field is Windows-specific and omitted for Linux containers.
     */
    instance_id?: string;
}
/** PidsStats contains Linux-specific stats of a container's process-IDs (PIDs).  This type is Linux-specific and omitted for Windows containers. */
export interface ContainerPidsStats {
    /** Current is the number of PIDs in the cgroup. */
    current?: U64;
    /** Limit is the hard limit on the number of pids in the cgroup. A \"Limit\" of 0 means that there is no limit. */
    limit?: U64;
}
/**
 * StorageStats is the disk I/O stats for read/write on Windows.
 * This type is Windows-specific and omitted for Linux containers.
 */
export interface ContainerStorageStats {
    read_count_normalized?: U64;
    read_size_bytes?: U64;
    write_count_normalized?: U64;
    write_size_bytes?: U64;
}
export interface Conversion {
    /** reference on the server. */
    local: string;
    /** reference in the container. */
    container: string;
}
/**
 * Creates a new action with given `name` and the configuration
 * of the action at the given `id`. Response: [Action].
 */
export interface CopyAction {
    /** The name of the new action. */
    name: string;
    /** The id of the action to copy. */
    id: string;
}
/**
 * Creates a new alerter with given `name` and the configuration
 * of the alerter at the given `id`. Response: [Alerter].
 */
export interface CopyAlerter {
    /** The name of the new alerter. */
    name: string;
    /** The id of the alerter to copy. */
    id: string;
}
/**
 * Creates a new build with given `name` and the configuration
 * of the build at the given `id`. Response: [Build].
 */
export interface CopyBuild {
    /** The name of the new build. */
    name: string;
    /** The id of the build to copy. */
    id: string;
}
/**
 * Creates a new builder with given `name` and the configuration
 * of the builder at the given `id`. Response: [Builder]
 */
export interface CopyBuilder {
    /** The name of the new builder. */
    name: string;
    /** The id of the builder to copy. */
    id: string;
}
/**
 * Creates a new deployment with given `name` and the configuration
 * of the deployment at the given `id`. Response: [Deployment]
 */
export interface CopyDeployment {
    /** The name of the new deployment. */
    name: string;
    /** The id of the deployment to copy. */
    id: string;
}
/**
 * Creates a new procedure with given `name` and the configuration
 * of the procedure at the given `id`. Response: [Procedure].
 */
export interface CopyProcedure {
    /** The name of the new procedure. */
    name: string;
    /** The id of the procedure to copy. */
    id: string;
}
/**
 * Creates a new repo with given `name` and the configuration
 * of the repo at the given `id`. Response: [Repo].
 */
export interface CopyRepo {
    /** The name of the new repo. */
    name: string;
    /** The id of the repo to copy. */
    id: string;
}
/**
 * Creates a new sync with given `name` and the configuration
 * of the sync at the given `id`. Response: [ResourceSync].
 */
export interface CopyResourceSync {
    /** The name of the new sync. */
    name: string;
    /** The id of the sync to copy. */
    id: string;
}
/**
 * Creates a new server with given `name` and the configuration
 * of the server at the given `id`. Response: [Server].
 */
export interface CopyServer {
    /** The name of the new server. */
    name: string;
    /** The id of the server to copy. */
    id: string;
    /** Initial public key to assign to Server. */
    public_key?: string;
}
/**
 * Creates a new stack with given `name` and the configuration
 * of the stack at the given `id`. Response: [Stack].
 */
export interface CopyStack {
    /** The name of the new stack. */
    name: string;
    /** The id of the stack to copy. */
    id: string;
}
/**
 * Creates a new Swarm with given `name` and the configuration
 * of the Swarm at the given `id`. Response: [Swarm].
 */
export interface CopySwarm {
    /** The name of the new swarm. */
    name: string;
    /** The id of the swarm to copy. */
    id: string;
}
/** Create a action. Response: [Action]. */
export interface CreateAction {
    /** The name given to newly created action. */
    name: string;
    /** Optional partial config to initialize the action with. */
    config?: _PartialActionConfig;
}
/** Create an alerter. Response: [Alerter]. */
export interface CreateAlerter {
    /** The name given to newly created alerter. */
    name: string;
    /** Optional partial config to initialize the alerter with. */
    config?: _PartialAlerterConfig;
}
/**
 * Create an api key for the calling user.
 * Response: [CreateApiKeyResponse].
 *
 * Note. After the response is served, there will be no way
 * to get the secret later.
 */
export interface CreateApiKey {
    /** The name for the api key. */
    name: string;
    /**
     * A unix timestamp in millseconds specifying api key expire time.
     * Default is 0, which means no expiry.
     */
    expires?: I64;
}
/**
 * Admin only method to create an api key for a service user.
 * Response: [CreateApiKeyResponse].
 */
export interface CreateApiKeyForServiceUser {
    /** Must be service user */
    user_id: string;
    /** The name for the api key */
    name: string;
    /**
     * A unix timestamp in millseconds specifying api key expire time.
     * Default is 0, which means no expiry.
     */
    expires?: I64;
}
/** Create a build. Response: [Build]. */
export interface CreateBuild {
    /** The name given to newly created build. */
    name: string;
    /** Optional partial config to initialize the build with. */
    config?: _PartialBuildConfig;
}
/** Partial representation of [BuilderConfig] */
export type PartialBuilderConfig = {
    type: "Url";
    params: _PartialUrlBuilderConfig;
} | {
    type: "Server";
    params: _PartialServerBuilderConfig;
} | {
    type: "Aws";
    params: _PartialAwsBuilderConfig;
};
/** Create a builder. Response: [Builder]. */
export interface CreateBuilder {
    /** The name given to newly created builder. */
    name: string;
    /** Optional partial config to initialize the builder with. */
    config?: PartialBuilderConfig;
}
/** Create a deployment. Response: [Deployment]. */
export interface CreateDeployment {
    /** The name given to newly created deployment. */
    name: string;
    /** Optional partial config to initialize the deployment with. */
    config?: _PartialDeploymentConfig;
}
/** Create a Deployment from an existing container. Response: [Deployment]. */
export interface CreateDeploymentFromContainer {
    /** The name or id of the existing container. */
    name: string;
    /** The server id or name on which container exists. */
    server: string;
}
/**
 * **Admin only.** Create a docker registry account.
 * Response: [DockerRegistryAccount].
 */
export interface CreateDockerRegistryAccount {
    account: _PartialDockerRegistryAccount;
}
/**
 * **Admin only.** Create a git provider account.
 * Response: [GitProviderAccount].
 */
export interface CreateGitProviderAccount {
    /**
     * The initial account config. Anything in the _id field will be ignored,
     * as this is generated on creation.
     */
    account: _PartialGitProviderAccount;
}
/**
 * **Admin only.** Create a local user.
 * Response: [User].
 *
 * Note. Not to be confused with /auth/SignUpLocalUser.
 * This method requires admin user credentials, and can
 * bypass disabled user registration.
 */
export interface CreateLocalUser {
    /** The username for the local user. */
    username: string;
    /** A password for the local user. */
    password: string;
}
/**
 * Create a docker network on the server.
 * Response: [Update]
 *
 * `docker network create {name}`
 */
export interface CreateNetwork {
    /** Server Id or name */
    server: string;
    /** The name of the network to create. */
    name: string;
}
/**
 * **Admin only.** Create a Server onboarding key.
 * Response: [CreateOnboardingKeyResponse].
 *
 * Note. The 'periphery_public_key' on default Server config will
 * be overridden with the actual public key once its generated by Periphery
 * as part of the onboarding flow.
 */
export interface CreateOnboardingKey {
    /** The name for the creation key */
    name: string;
    /**
     * A unix timestamp in millseconds specifying api key expire time.
     * Default is 0, which means no expiry.
     */
    expires?: I64;
    /**
     * Optionally specify an existing private key, otherwise
     * generate fresh key.
     */
    private_key?: string;
    /** Default tags to apply to Servers created using this key. */
    tags?: string[];
    /** Optional. New Servers copy this Server's config. */
    copy_server?: string;
    /** Optional. Whether to also create a Builder for the Server. */
    create_builder?: boolean;
}
/** The response for [CreateServerOnboardingKey] */
export interface CreateOnboardingKeyResponse {
    /** pkcs8 encoded private key */
    private_key: string;
    /** The created ServerOnboardingKey */
    created: OnboardingKey;
}
/** Create a procedure. Response: [Procedure]. */
export interface CreateProcedure {
    /** The name given to newly created build. */
    name: string;
    /** Optional partial config to initialize the procedure with. */
    config?: _PartialProcedureConfig;
}
/** Create a repo. Response: [Repo]. */
export interface CreateRepo {
    /** The name given to newly created repo. */
    name: string;
    /** Optional partial config to initialize the repo with. */
    config?: _PartialRepoConfig;
}
/** Create a sync. Response: [ResourceSync]. */
export interface CreateResourceSync {
    /** The name given to newly created sync. */
    name: string;
    /** Optional partial config to initialize the sync with. */
    config?: _PartialResourceSyncConfig;
}
/** Create a server. Response: [Server]. */
export interface CreateServer {
    /** The name given to newly created server. */
    name: string;
    /** Optional partial config to initialize the server with. */
    config?: _PartialServerConfig;
    /** Initial public key to assign to Server. */
    public_key?: string;
}
/**
 * **Admin only.** Create a service user.
 * Response: [User].
 */
export interface CreateServiceUser {
    /** The username for the service user. */
    username: string;
    /** A description for the service user. */
    description: string;
}
/** Create a stack. Response: [Stack]. */
export interface CreateStack {
    /** The name given to newly created stack. */
    name: string;
    /** Optional partial config to initialize the stack with. */
    config?: _PartialStackConfig;
}
/** Create a Swarm. Response: [Swarm]. */
export interface CreateSwarm {
    /** The name given to newly created swarm. */
    name: string;
    /** Optional partial config to initialize the swarm with. */
    config?: _PartialSwarmConfig;
}
/** Create a tag. Response: [Tag]. */
export interface CreateTag {
    /** The name of the tag. */
    name: string;
    /** Tag color. Default: Slate. */
    color?: TagColor;
}
/**
 * Create a Terminal.
 * Requires minimum Read + Terminal permission on the target Resource.
 * Response: [NoData]
 */
export interface CreateTerminal {
    /** A name for the Terminal session. */
    name: string;
    /** The target to create terminal for */
    target: TerminalTarget;
    /**
     * The shell command (eg `bash`) to init the shell.
     *
     * Default:
     * - Server: Configured on each Periphery
     * - ContainerExec: `sh`
     * - Attach: unused
     */
    command?: string;
    /**
     * For container terminals, choose 'exec' or 'attach'.
     *
     * Default
     * - Server: ignored
     * - Container / Stack / Deployment: `exec`
     */
    mode?: ContainerTerminalMode;
    /** Default: `Never` */
    recreate?: TerminalRecreateMode;
}
/** **Admin only.** Create a user group. Response: [UserGroup] */
export interface CreateUserGroup {
    /** The name to assign to the new UserGroup */
    name: string;
}
/** **Admin only.** Create variable. Response: [Variable]. */
export interface CreateVariable {
    /** The name of the variable to create. */
    name: string;
    /** The initial value of the variable. defualt: "". */
    value?: string;
    /** The initial value of the description. default: "". */
    description?: string;
    /** Whether to make this a secret variable. */
    is_secret?: boolean;
}
/** Configuration for a Custom alerter endpoint. */
export interface CustomAlerterEndpoint {
    /** The http/s endpoint to send the POST to */
    url: string;
}
/**
 * Deletes the action at the given id, and returns the deleted action.
 * Response: [Action]
 */
export interface DeleteAction {
    /** The id or name of the action to delete. */
    id: string;
}
/**
 * Deletes the alerter at the given id, and returns the deleted alerter.
 * Response: [Alerter]
 */
export interface DeleteAlerter {
    /** The id or name of the alerter to delete. */
    id: string;
}
/**
 * Delete all Terminals on the Server.
 * Response: [NoData]
 */
export interface DeleteAllTerminals {
    /** Server Id or name */
    server: string;
}
/**
 * Delete an api key for the calling user.
 * Response: [NoData]
 */
export interface DeleteApiKey {
    /** The key which the user intends to delete. */
    key: string;
}
/**
 * Admin only method to delete an api key for a service user.
 * Response: [NoData].
 */
export interface DeleteApiKeyForServiceUser {
    key: string;
}
/**
 * Deletes the build at the given id, and returns the deleted build.
 * Response: [Build]
 */
export interface DeleteBuild {
    /** The id or name of the build to delete. */
    id: string;
}
/**
 * Deletes the builder at the given id, and returns the deleted builder.
 * Response: [Builder]
 */
export interface DeleteBuilder {
    /** The id or name of the builder to delete. */
    id: string;
}
/**
 * Deletes the deployment at the given id, and returns the deleted deployment.
 * Response: [Deployment].
 *
 * Note. If the associated container is running, it will be deleted as part of
 * the deployment clean up.
 */
export interface DeleteDeployment {
    /** The id or name of the deployment to delete. */
    id: string;
}
/**
 * **Admin only.** Delete a docker registry account.
 * Response: [DockerRegistryAccount].
 */
export interface DeleteDockerRegistryAccount {
    /** The id of the docker registry account to delete */
    id: string;
}
/**
 * **Admin only.** Delete a git provider account.
 * Response: [DeleteGitProviderAccountResponse].
 */
export interface DeleteGitProviderAccount {
    /** The id of the git provider to delete */
    id: string;
}
/**
 * Delete a docker image.
 * Response: [Update]
 */
export interface DeleteImage {
    /** Id or name. */
    server: string;
    /** The name of the image to delete. */
    name: string;
}
/**
 * Delete a docker network.
 * Response: [Update]
 */
export interface DeleteNetwork {
    /** Id or name. */
    server: string;
    /** The name of the network to delete. */
    name: string;
}
/**
 * **Admin only.** Delete an onboarding key.
 * Response: The deleted [OnboardingKey].
 */
export interface DeleteOnboardingKey {
    public_key: string;
}
/**
 * Deletes the procedure at the given id, and returns the deleted procedure.
 * Response: [Procedure]
 */
export interface DeleteProcedure {
    /** The id or name of the procedure to delete. */
    id: string;
}
/**
 * Deletes the repo at the given id, and returns the deleted repo.
 * Response: [Repo]
 */
export interface DeleteRepo {
    /** The id or name of the repo to delete. */
    id: string;
}
/**
 * Deletes the sync at the given id, and returns the deleted sync.
 * Response: [ResourceSync]
 */
export interface DeleteResourceSync {
    /** The id or name of the sync to delete. */
    id: string;
}
/**
 * Deletes the server at the given id, and returns the deleted server.
 * Response: [Server]
 */
export interface DeleteServer {
    /** The id or name of the server to delete. */
    id: string;
}
/**
 * Deletes the stack at the given id, and returns the deleted stack.
 * Response: [Stack]
 */
export interface DeleteStack {
    /** The id or name of the stack to delete. */
    id: string;
}
/**
 * Deletes the Swarm at the given id, and returns the deleted Swarm.
 * Response: [Swarm]
 */
export interface DeleteSwarm {
    /** The id or name of the swarm to delete. */
    id: string;
}
/**
 * Delete a tag, and return the deleted tag. Response: [Tag].
 *
 * Note. Will also remove this tag from all attached resources.
 */
export interface DeleteTag {
    /** The id of the tag to delete. */
    id: string;
}
/**
 * Delete a terminal.
 * Response: [NoData]
 */
export interface DeleteTerminal {
    /** Server / Container / Stack / Deployment */
    target: TerminalTarget;
    /** The name of the Terminal to delete. */
    terminal: string;
}
/**
 * **Admin only**. Delete a user.
 * Admins can delete any non-admin user.
 * Only Super Admin can delete an admin.
 * No users can delete a Super Admin user.
 * User cannot delete themselves.
 * Response: [NoData].
 */
export interface DeleteUser {
    /** User id or username */
    user: string;
}
/** **Admin only.** Delete a user group. Response: [UserGroup] */
export interface DeleteUserGroup {
    /** The id of the UserGroup */
    id: string;
}
/** **Admin only.** Delete a variable. Response: [Variable]. */
export interface DeleteVariable {
    name: string;
}
/**
 * Delete a docker volume.
 * Response: [Update]
 */
export interface DeleteVolume {
    /** Id or name. */
    server: string;
    /** The name of the volume to delete. */
    name: string;
}
/**
 * Deploys the container for the target deployment. Response: [Update].
 *
 * 1. Pulls the image onto the target server.
 * 2. If the container is already running,
 * it will be stopped and removed using `docker container rm ${container_name}`.
 * 3. The container will be run using `docker run {...params}`,
 * where params are determined by the deployment's configuration.
 */
export interface Deploy {
    /** Name or id */
    deployment: string;
    /**
     * Override the default termination signal specified in the deployment.
     * Only used when deployment needs to be taken down before redeploy.
     */
    stop_signal?: TerminationSignal;
    /**
     * Override the default termination max time.
     * Only used when deployment needs to be taken down before redeploy.
     */
    stop_time?: number;
}
/** Deploys the target stack. `docker compose up`. Response: [Update] */
export interface DeployStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only deploy specific services.
     * If empty, will deploy all services.
     *
     * Note. For Swarm mode Stacks, this field is not supported and will be ignored.
     */
    services?: string[];
    /**
     * Override the default termination max time.
     * Only used if the stack needs to be taken down first.
     */
    stop_time?: number;
}
/**
 * Checks deployed contents vs latest contents,
 * and only if any changes found
 * will `docker compose up`. Response: [Update]
 */
export interface DeployStackIfChanged {
    /** Id or name */
    stack: string;
    /**
     * Override the default termination max time.
     * Only used if the stack needs to be taken down first.
     */
    stop_time?: number;
}
/**
 * Stops and destroys the container on the target server.
 * Reponse: [Update].
 *
 * 1. The container is stopped and removed using `docker container rm ${container_name}`.
 */
export interface DestroyContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
    /** Override the default termination signal. */
    signal?: TerminationSignal;
    /** Override the default termination max time. */
    time?: number;
}
/**
 * Stops and destroys the container for the target deployment.
 * Reponse: [Update].
 *
 * 1. The container is stopped and removed using `docker container rm ${container_name}`.
 */
export interface DestroyDeployment {
    /** Name or id. */
    deployment: string;
    /** Override the default termination signal specified in the deployment. */
    signal?: TerminationSignal;
    /** Override the default termination max time. */
    time?: number;
}
/** Destoys the target stack. `docker compose down`. Response: [Update] */
export interface DestroyStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only destroy specific services.
     * If empty, will destroy all services.
     */
    services?: string[];
    /** Pass `--remove-orphans` */
    remove_orphans?: boolean;
    /** Override the default termination max time. */
    stop_time?: number;
}
/** Configuration for a Discord alerter. */
export interface DiscordAlerterEndpoint {
    /** The Discord webhook url */
    url: string;
}
/** Standard docker lists available from a Server. */
export interface DockerLists {
    containers: ContainerListItem[];
    networks: NetworkListItem[];
    images: ImageListItem[];
    volumes: VolumeListItem[];
    projects: ComposeProject[];
}
export interface EnvironmentVar {
    variable: string;
    value: string;
}
/**
 * Exchange a single use exchange token (safe for transport in url query)
 * for a jwt.
 * Response: [ExchangeForJwtResponse].
 */
export interface ExchangeForJwt {
    /** The 'exchange token' */
    token: string;
}
/** Execute a terminal command on the given server. */
export interface ExecuteTerminalBody {
    /** The target to create terminal for. */
    target: TerminalTarget;
    /**
     * Terminal name to connect to.
     * If it may not exist yet, also pass 'init' params
     * to include initialization.
     * Default: Depends on target.
     */
    terminal?: string;
    /** The command to execute. */
    command: string;
    /**
     * Pass to init the terminal session
     * for when the terminal doesn't already exist.
     */
    init?: InitTerminal;
}
/**
 * Get pretty formatted monrun sync toml for all resources
 * which the user has permissions to view.
 * Response: [TomlResponse].
 */
export interface ExportAllResourcesToToml {
    /**
     * Whether to include any resources (servers, stacks, etc.)
     * in the exported contents.
     * Default: `true`
     */
    include_resources: boolean;
    /**
     * Filter resources by tag.
     * Accepts tag name or id. Empty array will not filter by tag.
     */
    tags?: string[];
    /**
     * Whether to include variables in the exported contents.
     * Default: false
     */
    include_variables?: boolean;
    /**
     * Whether to include user groups in the exported contents.
     * Default: false
     */
    include_user_groups?: boolean;
}
/**
 * Get pretty formatted monrun sync toml for specific resources and user groups.
 * Response: [TomlResponse].
 */
export interface ExportResourcesToToml {
    /** The targets to include in the export. */
    targets?: ResourceTarget[];
    /** The user group names or ids to include in the export. */
    user_groups?: string[];
    /** Whether to include variables */
    include_variables?: boolean;
}
/**
 * **Admin only.**
 * Find a user.
 * Response: [FindUserResponse]
 */
export interface FindUser {
    /** Id or username */
    user: string;
}
/** Statistics sample for a container. */
export interface FullContainerStats {
    /** Name of the container */
    name: string;
    /** ID of the container */
    id?: string;
    /**
     * Date and time at which this sample was collected.
     * The value is formatted as [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) with nano-seconds.
     */
    read?: string;
    /**
     * Date and time at which this first sample was collected.
     * This field is not propagated if the \"one-shot\" option is set.
     * If the \"one-shot\" option is set, this field may be omitted, empty,
     * or set to a default date (`0001-01-01T00:00:00Z`).
     * The value is formatted as [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) with nano-seconds.
     */
    preread?: string;
    /**
     * PidsStats contains Linux-specific stats of a container's process-IDs (PIDs).
     * This type is Linux-specific and omitted for Windows containers.
     */
    pids_stats?: ContainerPidsStats;
    /**
     * BlkioStats stores all IO service stats for data read and write.
     * This type is Linux-specific and holds many fields that are specific to cgroups v1.
     * On a cgroup v2 host, all fields other than `io_service_bytes_recursive` are omitted or `null`.
     * This type is only populated on Linux and omitted for Windows containers.
     */
    blkio_stats?: ContainerBlkioStats;
    /**
     * The number of processors on the system.
     * This field is Windows-specific and always zero for Linux containers.
     */
    num_procs?: number;
    storage_stats?: ContainerStorageStats;
    cpu_stats?: ContainerCpuStats;
    precpu_stats?: ContainerCpuStats;
    memory_stats?: ContainerMemoryStats;
    /** Network statistics for the container per interface.  This field is omitted if the container has no networking enabled. */
    networks?: Record<string, ContainerNetworkStats>;
}
/** Get a specific action. Response: [Action]. */
export interface GetAction {
    /** Id or name */
    action: string;
}
/** Get current action state for the action. Response: [ActionActionState]. */
export interface GetActionActionState {
    /** Id or name */
    action: string;
}
/**
 * Gets a summary of data relating to all actions.
 * Response: [GetActionsSummaryResponse].
 */
export interface GetActionsSummary {
}
/** Response for [GetActionsSummary]. */
export interface GetActionsSummaryResponse {
    /** The total number of actions. */
    total: number;
    /** The number of actions with Ok state. */
    ok: number;
    /** The number of actions currently running. */
    running: number;
    /** The number of actions with failed state. */
    failed: number;
    /** The number of actions with unknown state. */
    unknown: number;
}
/** Get an alert: Response: [Alert]. */
export interface GetAlert {
    id: string;
}
/** Get a specific alerter. Response: [Alerter]. */
export interface GetAlerter {
    /** Id or name */
    alerter: string;
}
/**
 * Gets a summary of data relating to all alerters.
 * Response: [GetAlertersSummaryResponse].
 */
export interface GetAlertersSummary {
}
/** Response for [GetAlertersSummary]. */
export interface GetAlertersSummaryResponse {
    total: number;
}
/** Get a specific build. Response: [Build]. */
export interface GetBuild {
    /** Id or name */
    build: string;
}
/** Get current action state for the build. Response: [BuildActionState]. */
export interface GetBuildActionState {
    /** Id or name */
    build: string;
}
/**
 * Gets summary and timeseries breakdown of the last months build count / time for charting.
 * Response: [GetBuildMonthlyStatsResponse].
 *
 * Note. This method is paginated. One page is 30 days of data.
 * Query for older pages by incrementing the page, starting at 0.
 */
export interface GetBuildMonthlyStats {
    /**
     * Query for older data by incrementing the page.
     * `page: 0` is the default, and will return the most recent data.
     */
    page?: number;
}
/** Response for [GetBuildMonthlyStats]. */
export interface GetBuildMonthlyStatsResponse {
    total_time: number;
    total_count: number;
    days: BuildStatsDay[];
}
/** Get a specific builder by id or name. Response: [Builder]. */
export interface GetBuilder {
    /** Id or name */
    builder: string;
}
/**
 * Gets a summary of data relating to all builders.
 * Response: [GetBuildersSummaryResponse].
 */
export interface GetBuildersSummary {
}
/** Response for [GetBuildersSummary]. */
export interface GetBuildersSummaryResponse {
    /** The total number of builders. */
    total: number;
}
/**
 * Gets a summary of data relating to all builds.
 * Response: [GetBuildsSummaryResponse].
 */
export interface GetBuildsSummary {
}
/** Response for [GetBuildsSummary]. */
export interface GetBuildsSummaryResponse {
    /** The total number of builds in Komodo. */
    total: number;
    /** The number of builds with Ok state. */
    ok: number;
    /** The number of builds with Failed state. */
    failed: number;
    /** The number of builds currently building. */
    building: number;
    /** The number of builds with unknown state. */
    unknown: number;
}
/**
 * Get the container log's tail, split by stdout/stderr.
 * Response: [Log].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface GetContainerLog {
    /** Id or name */
    server: string;
    /** The container name */
    container: string;
    /**
     * The number of lines of the log tail to include.
     * Default: 100.
     * Max: 5000.
     */
    tail: U64;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Get info about the core api configuration.
 * Response: [GetCoreInfoResponse].
 */
export interface GetCoreInfo {
}
/** Response for [GetCoreInfo]. */
export interface GetCoreInfoResponse {
    /** The title assigned to this core api. */
    title: string;
    /** The monitoring interval of this core api. */
    monitoring_interval: Timelength;
    /** The webhook base url. */
    webhook_base_url: string;
    /** Whether transparent mode is enabled, which gives all users read access to all resources. */
    transparent_mode: boolean;
    /** Whether UI write access should be disabled */
    ui_write_disabled: boolean;
    /** Whether non admins can create resources */
    disable_non_admin_create: boolean;
    /** Whether confirm dialog should be disabled */
    disable_confirm_dialog: boolean;
    /** Whether to disable websocket automatic reconnect. */
    disable_websocket_reconnect: boolean;
    /** Whether to enable fancy toml highlighting. */
    enable_fancy_toml: boolean;
    /** TZ identifier Core is using, if manually set. */
    timezone: string;
    /** Default public key allowing this Core to authenticate to Periphery agents. */
    public_key: string;
}
/** Get a specific deployment by name or id. Response: [Deployment]. */
export interface GetDeployment {
    /** Id or name */
    deployment: string;
}
/**
 * Get current action state for the deployment.
 * Response: [DeploymentActionState].
 */
export interface GetDeploymentActionState {
    /** Id or name */
    deployment: string;
}
/**
 * Get the container, including image / status, of the target deployment.
 * Response: [GetDeploymentContainerResponse].
 *
 * Note. This does not hit the server directly. The status comes from an
 * in memory cache on the core, which hits the server periodically
 * to keep it up to date.
 */
export interface GetDeploymentContainer {
    /** Id or name */
    deployment: string;
}
/** Response for [GetDeploymentContainer]. */
export interface GetDeploymentContainerResponse {
    state: DeploymentState;
    container?: ContainerListItem;
}
/**
 * Get the deployment log's tail, split by stdout/stderr.
 * Response: [Log].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface GetDeploymentLog {
    /** Id or name */
    deployment: string;
    /**
     * The number of lines of the log tail to include.
     * Default: 100.
     * Max: 5000.
     */
    tail: U64;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Get the deployment container's stats using `docker stats`.
 * Response: [GetDeploymentStatsResponse].
 *
 * Note. This call will hit the underlying server directly for most up to date stats.
 */
export interface GetDeploymentStats {
    /** Id or name */
    deployment: string;
}
/**
 * Gets a summary of data relating to all deployments.
 * Response: [GetDeploymentsSummaryResponse].
 */
export interface GetDeploymentsSummary {
}
/** Response for [GetDeploymentsSummary]. */
export interface GetDeploymentsSummaryResponse {
    /** The total number of Deployments */
    total: I64;
    /** The number of Deployments with Running state */
    running: I64;
    /** The number of Deployments with Stopped or Paused state */
    stopped: I64;
    /** The number of Deployments with NotDeployed state */
    not_deployed: I64;
    /** The number of Deployments with Restarting or Dead or Created (other) state */
    unhealthy: I64;
    /** The number of Deployments with Unknown state */
    unknown: I64;
}
/**
 * Gets a summary of data relating to all containers.
 * Response: [GetDockerContainersSummaryResponse].
 */
export interface GetDockerContainersSummary {
}
/** Response for [GetDockerContainersSummary] */
export interface GetDockerContainersSummaryResponse {
    /** The total number of Containers */
    total: number;
    /** The number of Containers with Running state */
    running: number;
    /** The number of Containers with Stopped or Paused or Created state */
    stopped: number;
    /** The number of Containers with Restarting or Dead state */
    unhealthy: number;
    /** The number of Containers with Unknown state */
    unknown: number;
}
/**
 * Get a specific docker registry account.
 * Response: [GetDockerRegistryAccountResponse].
 */
export interface GetDockerRegistryAccount {
    id: string;
}
/**
 * Get a specific git provider account.
 * Response: [GetGitProviderAccountResponse].
 */
export interface GetGitProviderAccount {
    id: string;
}
/**
 * Paginated endpoint serving historical (timeseries) server stats for graphing.
 * Response: [GetHistoricalServerStatsResponse].
 */
export interface GetHistoricalServerStats {
    /** Id or name */
    server: string;
    /** The granularity of the data. */
    granularity: Timelength;
    /**
     * Page of historical data. Default is 0, which is the most recent data.
     * Use with the `next_page` field of the response.
     */
    page?: number;
}
/** System stats stored on the database. */
export interface SystemStatsRecord {
    /** Unix timestamp in milliseconds */
    ts: I64;
    /** Server id */
    sid: string;
    /** Cpu usage percentage */
    cpu_perc: number;
    /** Load average (1m, 5m, 15m) */
    load_average?: SystemLoadAverage;
    /** Memory used in GB */
    mem_used_gb: number;
    /** Total memory in GB */
    mem_total_gb: number;
    /** Disk used in GB */
    disk_used_gb: number;
    /** Total disk size in GB */
    disk_total_gb: number;
    /** Breakdown of individual disks, including their usage, total size, and mount point */
    disks: SingleDiskUsage[];
    /** Total network ingress in bytes */
    network_ingress_bytes?: number;
    /** Total network egress in bytes */
    network_egress_bytes?: number;
}
/** Response to [GetHistoricalServerStats]. */
export interface GetHistoricalServerStatsResponse {
    /** The timeseries page of data. */
    stats: SystemStatsRecord[];
    /** If there is a next page of data, pass this to `page` to get it. */
    next_page?: number;
}
/**
 * Non authenticated route to see the available options
 * users have to login to Komodo, eg. local auth, github, google.
 * Response: [GetLoginOptionsResponse].
 */
export interface GetLoginOptions {
}
/** The response for [GetLoginOptions]. */
export interface GetLoginOptionsResponse {
    /** Whether local auth is enabled. */
    local: boolean;
    /** Whether github login is enabled. */
    github: boolean;
    /** Whether google login is enabled. */
    google: boolean;
    /** Whether OIDC login is enabled. */
    oidc: boolean;
    /** Whether user registration (Sign Up) has been disabled */
    registration_disabled: boolean;
}
/**
 * Get the Periphery information of the target server,
 * including the Periphery version and public key.
 * Response: [PeripheryInformation].
 */
export interface GetPeripheryInformation {
    /** Id or name */
    server: string;
}
/**
 * Gets the calling user's permission level on a specific resource.
 * Factors in any UserGroup's permissions they may be a part of.
 * Response: [PermissionLevel]
 */
export interface GetPermission {
    /** The target to get user permission on. */
    target: ResourceTarget;
}
/** Get a specific procedure. Response: [Procedure]. */
export interface GetProcedure {
    /** Id or name */
    procedure: string;
}
/** Get current action state for the procedure. Response: [ProcedureActionState]. */
export interface GetProcedureActionState {
    /** Id or name */
    procedure: string;
}
/**
 * Gets a summary of data relating to all procedures.
 * Response: [GetProceduresSummaryResponse].
 */
export interface GetProceduresSummary {
}
/** Response for [GetProceduresSummary]. */
export interface GetProceduresSummaryResponse {
    /** The total number of procedures. */
    total: number;
    /** The number of procedures with Ok state. */
    ok: number;
    /** The number of procedures currently running. */
    running: number;
    /** The number of procedures with failed state. */
    failed: number;
    /** The number of procedures with unknown state. */
    unknown: number;
}
/** Get a specific repo. Response: [Repo]. */
export interface GetRepo {
    /** Id or name */
    repo: string;
}
/** Get current action state for the repo. Response: [RepoActionState]. */
export interface GetRepoActionState {
    /** Id or name */
    repo: string;
}
/**
 * Gets a summary of data relating to all repos.
 * Response: [GetReposSummaryResponse].
 */
export interface GetReposSummary {
}
/** Response for [GetReposSummary] */
export interface GetReposSummaryResponse {
    /** The total number of repos */
    total: number;
    /** The number of repos with Ok state. */
    ok: number;
    /** The number of repos currently cloning. */
    cloning: number;
    /** The number of repos currently pulling. */
    pulling: number;
    /** The number of repos currently building. */
    building: number;
    /** The number of repos with failed state. */
    failed: number;
    /** The number of repos with unknown state. */
    unknown: number;
}
/** Find the attached resource for a container. Either Deployment or Stack. Response: [GetResourceMatchingContainerResponse]. */
export interface GetResourceMatchingContainer {
    /** Id or name */
    server: string;
    /** The container name */
    container: string;
}
/** Response for [GetResourceMatchingContainer]. Resource is either Deployment, Stack, or None. */
export interface GetResourceMatchingContainerResponse {
    resource?: ResourceTarget;
}
/** Get a specific sync. Response: [ResourceSync]. */
export interface GetResourceSync {
    /** Id or name */
    sync: string;
}
/** Get current action state for the sync. Response: [ResourceSyncActionState]. */
export interface GetResourceSyncActionState {
    /** Id or name */
    sync: string;
}
/**
 * Gets a summary of data relating to all syncs.
 * Response: [GetResourceSyncsSummaryResponse].
 */
export interface GetResourceSyncsSummary {
}
/** Response for [GetResourceSyncsSummary] */
export interface GetResourceSyncsSummaryResponse {
    /** The total number of syncs */
    total: number;
    /** The number of syncs with Ok state. */
    ok: number;
    /** The number of syncs currently syncing. */
    syncing: number;
    /** The number of syncs with pending updates */
    pending: number;
    /** The number of syncs with failed state. */
    failed: number;
    /** The number of syncs with unknown state. */
    unknown: number;
}
/** Get a specific server. Response: [Server]. */
export interface GetServer {
    /** Id or name */
    server: string;
}
/** Get current action state for the servers. Response: [ServerActionState]. */
export interface GetServerActionState {
    /** Id or name */
    server: string;
}
/** Get the state of the target server. Response: [GetServerStateResponse]. */
export interface GetServerState {
    /** Id or name */
    server: string;
}
/** The response for [GetServerState]. */
export interface GetServerStateResponse {
    /** The server status. */
    status: ServerState;
}
/**
 * Gets a summary of data relating to all servers.
 * Response: [GetServersSummaryResponse].
 */
export interface GetServersSummary {
}
/** Response for [GetServersSummary]. */
export interface GetServersSummaryResponse {
    /** The total number of servers. */
    total: I64;
    /** The number of healthy (`status: OK`) servers. */
    healthy: I64;
    /** The number of servers with warnings (e.g., version mismatch). */
    warning: I64;
    /** The number of unhealthy servers. */
    unhealthy: I64;
    /** The number of disabled servers. */
    disabled: I64;
}
/** Get a specific stack. Response: [Stack]. */
export interface GetStack {
    /** Id or name */
    stack: string;
}
/** Get current action state for the stack. Response: [StackActionState]. */
export interface GetStackActionState {
    /** Id or name */
    stack: string;
}
/**
 * Get a stack's logs. Filter down included services. Response: [GetStackLogResponse].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface GetStackLog {
    /** Id or name */
    stack: string;
    /**
     * Filter the logs to only ones from specific services.
     * If empty, will include logs from all services.
     */
    services: string[];
    /**
     * The number of lines of the log tail to include.
     * Default: 100.
     * Max: 5000.
     */
    tail: U64;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Gets a summary of data relating to all syncs.
 * Response: [GetStacksSummaryResponse].
 */
export interface GetStacksSummary {
}
/** Response for [GetStacksSummary] */
export interface GetStacksSummaryResponse {
    /** The total number of stacks */
    total: number;
    /** The number of stacks with Running state. */
    running: number;
    /** The number of stacks with Stopped or Paused state. */
    stopped: number;
    /** The number of stacks with Down state. */
    down: number;
    /** The number of stacks with Unhealthy or Restarting or Dead or Created or Removing state. */
    unhealthy: number;
    /** The number of stacks with Unknown state. */
    unknown: number;
}
/** Get a specific swarm. Response: [Swarm]. */
export interface GetSwarm {
    /** Id or name */
    swarm: string;
}
/** Get current action state for the swarm. Response: [SwarmActionState]. */
export interface GetSwarmActionState {
    /** Id or name */
    swarm: string;
}
/**
 * Get a swarm service's logs. Response: [GetSwarmServiceLogResponse].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface GetSwarmServiceLog {
    /** Id or name */
    swarm: string;
    /** Select the swarm service to get logs for. */
    service: string;
    /**
     * The number of lines of the log tail to include.
     * Default: 100.
     * Max: 5000.
     */
    tail: U64;
    /** Enable `--timestamps` */
    timestamps?: boolean;
    /** Enable `--no-task-ids` */
    no_task_ids?: boolean;
    /** Enable `--no-resolve` */
    no_resolve?: boolean;
    /** Enable `--details` */
    details?: boolean;
}
/**
 * Gets a summary of data relating to all swarms.
 * Response: [GetSwarmsSummaryResponse].
 */
export interface GetSwarmsSummary {
}
/** Response for [GetSwarmsSummary] */
export interface GetSwarmsSummaryResponse {
    /** The total number of Swarms */
    total: number;
    /** The number of Swarms with Healthy state. */
    healthy: number;
    /** The number of Swarms with Unhealthy state */
    unhealthy: number;
    /** The number of Swarms with Unknown state */
    unknown: number;
}
/**
 * Get the system information of the target server.
 * Response: [SystemInformation].
 */
export interface GetSystemInformation {
    /** Id or name */
    server: string;
}
/**
 * Get the system stats on the target server. Response: [SystemStats].
 *
 * Note. This does not hit the server directly. The stats come from an
 * in memory cache on the core, which hits the server periodically
 * to keep it up to date.
 */
export interface GetSystemStats {
    /** Id or name */
    server: string;
}
/** Get data for a specific tag. Response [Tag]. */
export interface GetTag {
    /** Id or name */
    tag: string;
}
/**
 * Get all data for the target update.
 * Response: [Update].
 */
export interface GetUpdate {
    /** The update id. */
    id: string;
}
/**
 * Get the user extracted from the request headers.
 * Response: [User].
 */
export interface GetUser {
}
/**
 * Get a specific user group by name or id.
 * Response: [UserGroup].
 */
export interface GetUserGroup {
    /** Name or Id */
    user_group: string;
}
/**
 * Gets the username of a specific user.
 * Response: [GetUsernameResponse]
 */
export interface GetUsername {
    /** The id of the user. */
    user_id: string;
}
/** Response for [GetUsername]. */
export interface GetUsernameResponse {
    /** The username of the user. */
    username: string;
    /** An optional icon for the user. */
    avatar?: string;
}
/**
 * List all available global variables.
 * Response: [Variable]
 *
 * Note. For non admin users making this call,
 * secret variables will have their values obscured.
 */
export interface GetVariable {
    /** The name of the variable to get. */
    name: string;
}
/**
 * Get the version of the Komodo Core api.
 * Response: [GetVersionResponse].
 */
export interface GetVersion {
}
/** Response for [GetVersion]. */
export interface GetVersionResponse {
    /** The version of the core api. */
    version: string;
}
/**
 * **Admin only.** Trigger a global poll for image updates on Stacks and Deployments
 * with `poll_for_updates` or `auto_update` enabled.
 * Response: [Update]. Alias: `auto-update`.
 *
 * 1. `docker compose pull` any Stacks / Deployments with `poll_for_updates` or `auto_update` enabled. This will pick up any available updates.
 * 2. Redeploy Stacks / Deployments that have updates found and 'auto_update' enabled.
 */
export interface GlobalAutoUpdate {
}
/**
 * Inspect the docker container associated with the Deployment.
 * Response: [Container].
 */
export interface InspectDeploymentContainer {
    /** Id or name */
    deployment: string;
}
/** Inspect a docker container on the server. Response: [Container]. */
export interface InspectDockerContainer {
    /** Id or name */
    server: string;
    /** The container name */
    container: string;
}
/** Inspect a docker image on the server. Response: [Image]. */
export interface InspectDockerImage {
    /** Id or name */
    server: string;
    /** The image name */
    image: string;
}
/** Inspect a docker network on the server. Response: [InspectDockerNetworkResponse]. */
export interface InspectDockerNetwork {
    /** Id or name */
    server: string;
    /** The network name */
    network: string;
}
/** Inspect a docker volume on the server. Response: [Volume]. */
export interface InspectDockerVolume {
    /** Id or name */
    server: string;
    /** The volume name */
    volume: string;
}
/**
 * Inspect a docker container associated with a Stack.
 * Response: [Container].
 */
export interface InspectStackContainer {
    /** Id or name */
    stack: string;
    /** The service name to inspect */
    service: string;
}
/**
 * Inspect a swarm service associated with a Stack.
 * Response: [Container].
 */
export interface InspectStackSwarmService {
    /** Id or name */
    stack: string;
    /** The service name to inspect */
    service: string;
}
/**
 * Inspect information about the swarm.
 * Response: [SwarmInspectInfo].
 */
export interface InspectSwarm {
    /** Id or name */
    swarm: string;
}
/**
 * Inspect a config on the target Swarm.
 * Response: [InspectSwarmConfigResponse].
 */
export interface InspectSwarmConfig {
    /** Id or name */
    swarm: string;
    /** Swarm config ID or Name */
    config: string;
}
/**
 * Inspect a Swarm node.
 * Response: [SwarmNode].
 */
export interface InspectSwarmNode {
    /** Id or name */
    swarm: string;
    /** Node id */
    node: string;
}
/**
 * Inspect a Swarm secret.
 * Response: [SwarmSecret].
 */
export interface InspectSwarmSecret {
    /** Id or name */
    swarm: string;
    /** Secret id */
    secret: string;
}
/**
 * Inspect a Swarm service.
 * Response: [SwarmService].
 */
export interface InspectSwarmService {
    /** Id or name */
    swarm: string;
    /** Service id */
    service: string;
}
/**
 * Inspect a stack on the target Swarm.
 * Response: [SwarmStackLists].
 */
export interface InspectSwarmStack {
    /** Id or name */
    swarm: string;
    /** Swarm stack name */
    stack: string;
}
/**
 * Inspect a Swarm task.
 * Response: [SwarmTask].
 */
export interface InspectSwarmTask {
    /** Id or name */
    swarm: string;
    /** Task id */
    task: string;
}
export interface LatestCommit {
    hash: string;
    message: string;
}
/** List actions matching optional query. Response: [ListActionsResponse]. */
export interface ListActions {
    /** optional structured query to filter actions. */
    query?: ActionQuery;
}
/** List alerters matching optional query. Response: [ListAlertersResponse]. */
export interface ListAlerters {
    /** Structured query to filter alerters. */
    query?: AlerterQuery;
}
/**
 * Get a paginated list of alerts sorted by timestamp descending.
 * Response: [ListAlertsResponse].
 */
export interface ListAlerts {
    /**
     * Pass a custom mongo query to filter the alerts.
     *
     * ## Example JSON
     * ```json
     * {
     * "resolved": "false",
     * "level": "CRITICAL",
     * "$or": [
     * {
     * "target": {
     * "type": "Server",
     * "id": "6608bf89cb2a12b257ab6c09"
     * }
     * },
     * {
     * "target": {
     * "type": "Server",
     * "id": "660a5f60b74f90d5dae45fa3"
     * }
     * }
     * ]
     * }
     * ```
     * This will filter to only include open alerts that have CRITICAL level on those two servers.
     */
    query?: MongoDocument;
    /**
     * Retrieve older results by incrementing the page.
     * `page: 0` is default, and returns the most recent results.
     */
    page?: U64;
}
/** Response for [ListAlerts]. */
export interface ListAlertsResponse {
    alerts: Alert[];
    /**
     * If more alerts exist, the next page will be given here.
     * Otherwise it will be `null`
     */
    next_page?: I64;
}
/**
 * List all docker containers on the target servers.
 * Response: [ListDockerContainersResponse].
 */
export interface ListAllDockerContainers {
    /** Filter by server id or name. */
    servers?: string[];
    /** Filter by container name. */
    containers?: string[];
}
/**
 * Gets list of api keys for the calling user.
 * Response: [ListApiKeysResponse]
 */
export interface ListApiKeys {
}
/**
 * **Admin only.**
 * Gets list of api keys for the user.
 * Will still fail if you call for a user_id that isn't a service user.
 * Response: [ListApiKeysForServiceUserResponse]
 */
export interface ListApiKeysForServiceUser {
    /** Id or username */
    user: string;
}
/**
 * Retrieve versions of the build that were built in the past and available for deployment,
 * sorted by most recent first.
 * Response: [ListBuildVersionsResponse].
 */
export interface ListBuildVersions {
    /** Id or name */
    build: string;
    /** Filter to only include versions matching this major version. */
    major?: number;
    /** Filter to only include versions matching this minor version. */
    minor?: number;
    /** Filter to only include versions matching this patch version. */
    patch?: number;
    /** Limit the number of included results. Default is no limit. */
    limit?: I64;
}
/** List builders matching structured query. Response: [ListBuildersResponse]. */
export interface ListBuilders {
    query?: BuilderQuery;
}
/** List builds matching optional query. Response: [ListBuildsResponse]. */
export interface ListBuilds {
    /** optional structured query to filter builds. */
    query?: BuildQuery;
}
/**
 * Gets a list of existing values used as extra args across other builds.
 * Useful to offer suggestions. Response: [ListCommonBuildExtraArgsResponse]
 */
export interface ListCommonBuildExtraArgs {
    /** optional structured query to filter builds. */
    query?: BuildQuery;
}
/**
 * Gets a list of existing values used as extra args across other deployments.
 * Useful to offer suggestions. Response: [ListCommonDeploymentExtraArgsResponse]
 */
export interface ListCommonDeploymentExtraArgs {
    /** optional structured query to filter deployments. */
    query?: DeploymentQuery;
}
/**
 * Gets a list of existing values used as build extra args across other stacks.
 * Useful to offer suggestions. Response: [ListCommonStackBuildExtraArgsResponse]
 */
export interface ListCommonStackBuildExtraArgs {
    /** optional structured query to filter stacks. */
    query?: StackQuery;
}
/**
 * Gets a list of existing values used as extra args across other stacks.
 * Useful to offer suggestions. Response: [ListCommonStackExtraArgsResponse]
 */
export interface ListCommonStackExtraArgs {
    /** optional structured query to filter stacks. */
    query?: StackQuery;
}
/**
 * List all docker compose projects on the target server.
 * Response: [ListComposeProjectsResponse].
 */
export interface ListComposeProjects {
    /** Id or name */
    server: string;
}
/**
 * List deployments matching optional query.
 * Response: [ListDeploymentsResponse].
 */
export interface ListDeployments {
    /** optional structured query to filter deployments. */
    query?: DeploymentQuery;
}
/**
 * List all docker containers on the target server.
 * Response: [ListDockerContainersResponse].
 */
export interface ListDockerContainers {
    /** Id or name */
    server: string;
}
/** Get image history from the server. Response: [ListDockerImageHistoryResponse]. */
export interface ListDockerImageHistory {
    /** Id or name */
    server: string;
    /** The image name */
    image: string;
}
/**
 * List the docker images locally cached on the target server.
 * Response: [ListDockerImagesResponse].
 */
export interface ListDockerImages {
    /** Id or name */
    server: string;
}
/** List the docker networks on the server. Response: [ListDockerNetworksResponse]. */
export interface ListDockerNetworks {
    /** Id or name */
    server: string;
}
/**
 * List the docker registry providers available in Core / Periphery config files.
 * Response: [ListDockerRegistriesFromConfigResponse].
 *
 * Includes:
 * - registries in core config
 * - registries configured on builds, deployments
 * - registries on the optional Server or Builder
 */
export interface ListDockerRegistriesFromConfig {
    /**
     * Accepts an optional Server or Builder target to expand the core list with
     * providers available on that specific resource.
     */
    target?: ResourceTarget;
}
/**
 * List docker registry accounts matching optional query.
 * Response: [ListDockerRegistryAccountsResponse].
 */
export interface ListDockerRegistryAccounts {
    /** Optionally filter by accounts with a specific domain. */
    domain?: string;
    /** Optionally filter by accounts with a specific username. */
    username?: string;
}
/**
 * List all docker volumes on the target server.
 * Response: [ListDockerVolumesResponse].
 */
export interface ListDockerVolumes {
    /** Id or name */
    server: string;
}
/** List actions matching optional query. Response: [ListFullActionsResponse]. */
export interface ListFullActions {
    /** optional structured query to filter actions. */
    query?: ActionQuery;
}
/** List full alerters matching optional query. Response: [ListFullAlertersResponse]. */
export interface ListFullAlerters {
    /** Structured query to filter alerters. */
    query?: AlerterQuery;
}
/** List builders matching structured query. Response: [ListFullBuildersResponse]. */
export interface ListFullBuilders {
    query?: BuilderQuery;
}
/** List builds matching optional query. Response: [ListFullBuildsResponse]. */
export interface ListFullBuilds {
    /** optional structured query to filter builds. */
    query?: BuildQuery;
}
/**
 * List deployments matching optional query.
 * Response: [ListFullDeploymentsResponse].
 */
export interface ListFullDeployments {
    /** optional structured query to filter deployments. */
    query?: DeploymentQuery;
}
/** List procedures matching optional query. Response: [ListFullProceduresResponse]. */
export interface ListFullProcedures {
    /** optional structured query to filter procedures. */
    query?: ProcedureQuery;
}
/** List repos matching optional query. Response: [ListFullReposResponse]. */
export interface ListFullRepos {
    /** optional structured query to filter repos. */
    query?: RepoQuery;
}
/** List syncs matching optional query. Response: [ListFullResourceSyncsResponse]. */
export interface ListFullResourceSyncs {
    /** optional structured query to filter syncs. */
    query?: ResourceSyncQuery;
}
/** List servers matching optional query. Response: [ListFullServersResponse]. */
export interface ListFullServers {
    /** optional structured query to filter servers. */
    query?: ServerQuery;
}
/** List stacks matching optional query. Response: [ListFullStacksResponse]. */
export interface ListFullStacks {
    /** optional structured query to filter stacks. */
    query?: StackQuery;
}
/** List Swarms matching optional query. Response: [ListFullSwarmsResponse]. */
export interface ListFullSwarms {
    /** optional structured query to filter swarms. */
    query?: SwarmQuery;
}
/**
 * List git provider accounts matching optional query.
 * Response: [ListGitProviderAccountsResponse].
 */
export interface ListGitProviderAccounts {
    /** Optionally filter by accounts with a specific domain. */
    domain?: string;
    /** Optionally filter by accounts with a specific username. */
    username?: string;
}
/**
 * List the git providers available in Core / Periphery config files.
 * Response: [ListGitProvidersFromConfigResponse].
 *
 * Includes:
 * - providers in core config
 * - providers configured on builds, repos, syncs
 * - providers on the optional Server or Builder
 */
export interface ListGitProvidersFromConfig {
    /**
     * Accepts an optional Server or Builder target to expand the core list with
     * providers available on that specific resource.
     */
    target?: ResourceTarget;
}
/**
 * **Admin only.** Gets list of onboarding keys.
 * Response: [ListOnboardingKeysResponse]
 */
export interface ListOnboardingKeys {
}
/**
 * List permissions for the calling user.
 * Does not include any permissions on UserGroups they may be a part of.
 * Response: [ListPermissionsResponse]
 */
export interface ListPermissions {
}
/** List procedures matching optional query. Response: [ListProceduresResponse]. */
export interface ListProcedures {
    /** optional structured query to filter procedures. */
    query?: ProcedureQuery;
}
/** List repos matching optional query. Response: [ListReposResponse]. */
export interface ListRepos {
    /** optional structured query to filter repos. */
    query?: RepoQuery;
}
/** List syncs matching optional query. Response: [ListResourceSyncsResponse]. */
export interface ListResourceSyncs {
    /** optional structured query to filter syncs. */
    query?: ResourceSyncQuery;
}
/**
 * List configured schedules.
 * Response: [ListSchedulesResponse].
 */
export interface ListSchedules {
    /** Pass Vec of tag ids or tag names */
    tags?: string[];
    /** 'All' or 'Any' */
    tag_behavior?: TagQueryBehavior;
}
/**
 * List the available secrets from the core config.
 * Response: [ListSecretsResponse].
 */
export interface ListSecrets {
    /**
     * Accepts an optional Server or Builder target to expand the core list with
     * providers available on that specific resource.
     */
    target?: ResourceTarget;
}
/** List servers matching optional query. Response: [ListServersResponse]. */
export interface ListServers {
    /** optional structured query to filter servers. */
    query?: ServerQuery;
}
/** Lists a specific stacks services (the containers). Response: [ListStackServicesResponse]. */
export interface ListStackServices {
    /** Id or name */
    stack: string;
}
/** List stacks matching optional query. Response: [ListStacksResponse]. */
export interface ListStacks {
    /** optional structured query to filter stacks. */
    query?: StackQuery;
}
/**
 * List configs on the target Swarm.
 * Response: [ListSwarmConfigsResponse].
 */
export interface ListSwarmConfigs {
    /** Id or name */
    swarm: string;
}
/**
 * List nodes part of the target Swarm.
 * Response: [ListSwarmNodesResponse].
 */
export interface ListSwarmNodes {
    /** Id or name */
    swarm: string;
}
/**
 * List secrets on the target Swarm.
 * Response: [ListSwarmSecretsResponse].
 */
export interface ListSwarmSecrets {
    /** Id or name */
    swarm: string;
}
/**
 * List services on the target Swarm.
 * Response: [ListSwarmServicesResponse].
 */
export interface ListSwarmServices {
    /** Id or name */
    swarm: string;
}
/**
 * List stacks on the target Swarm.
 * Response: [ListSwarmStacksResponse].
 */
export interface ListSwarmStacks {
    /** Id or name */
    swarm: string;
}
/**
 * List tasks on the target Swarm.
 * Response: [ListSwarmTasksResponse].
 */
export interface ListSwarmTasks {
    /** Id or name */
    swarm: string;
}
/** List Swarms matching optional query. Response: [ListSwarmsResponse]. */
export interface ListSwarms {
    /** Optional structured query to filter Swarms. */
    query?: SwarmQuery;
}
/**
 * List the processes running on the target server.
 * Response: [ListSystemProcessesResponse].
 *
 * Note. This does not hit the server directly. The procedures come from an
 * in memory cache on the core, which hits the server periodically
 * to keep it up to date.
 */
export interface ListSystemProcesses {
    /** Id or name */
    server: string;
}
/**
 * List data for tags matching optional mongo query.
 * Response: [ListTagsResponse].
 */
export interface ListTags {
    query?: MongoDocument;
}
/**
 * List Terminals.
 * Response: [ListTerminalsResponse].
 */
export interface ListTerminals {
    /** Filter the Terminals returned by the Target. */
    target?: TerminalTarget;
    /** Return results with resource names instead of ids. */
    use_names?: boolean;
}
/**
 * Paginated endpoint for updates matching optional query.
 * More recent updates will be returned first.
 */
export interface ListUpdates {
    /** An optional mongo query to filter the updates. */
    query?: MongoDocument;
    /**
     * Page of updates. Default is 0, which is the most recent data.
     * Use with the `next_page` field of the response.
     */
    page?: number;
}
/** Minimal representation of an action performed by Komodo. */
export interface UpdateListItem {
    /** The id of the update */
    id: string;
    /** Which operation was run */
    operation: Operation;
    /** The starting time of the operation */
    start_ts: I64;
    /** Whether the operation was successful */
    success: boolean;
    /** The username of the user performing update */
    username: string;
    /**
     * The user id that triggered the update.
     *
     * Also can take these values for operations triggered automatically:
     * - `Procedure`: The operation was triggered as part of a procedure run
     * - `Github`: The operation was triggered by a github webhook
     * - `Auto Redeploy`: The operation (always `Deploy`) was triggered by an attached build finishing.
     */
    operator: string;
    /** The target resource to which this update refers */
    target: ResourceTarget;
    /**
     * The status of the update
     * - `Queued`
     * - `InProgress`
     * - `Complete`
     */
    status: UpdateStatus;
    /** An optional version on the update, ie build version or deployed version. */
    version?: Version;
    /** Some unstructured, operation specific data. Not for general usage. */
    other_data?: string;
}
/** Response for [ListUpdates]. */
export interface ListUpdatesResponse {
    /** The page of updates, sorted by timestamp descending. */
    updates: UpdateListItem[];
    /** If there is a next page of data, pass this to `page` to get it. */
    next_page?: number;
}
/**
 * List all user groups which user can see. Response: [ListUserGroupsResponse].
 *
 * Admins can see all user groups,
 * and users can see user groups to which they belong.
 */
export interface ListUserGroups {
}
/**
 * List permissions for a specific user. **Admin only**.
 * Response: [ListUserTargetPermissionsResponse]
 */
export interface ListUserTargetPermissions {
    /** Specify either a user or a user group. */
    user_target: UserTarget;
}
/**
 * **Admin only.**
 * Gets list of Komodo users.
 * Response: [ListUsersResponse]
 */
export interface ListUsers {
}
/**
 * List all available global variables.
 * Response: [ListVariablesResponse]
 *
 * Note. For non admin users making this call,
 * secret variables will have their values obscured.
 */
export interface ListVariables {
}
/**
 * Login as a local user. Will fail if the users credentials don't match
 * any local user.
 *
 * Note. This method is only available if the core api has `local_auth` enabled.
 */
export interface LoginLocalUser {
    /** The user's username */
    username: string;
    /** The user's password */
    password: string;
}
export interface NameAndId {
    name: string;
    id: string;
}
/** Configuration for a Ntfy alerter. */
export interface NtfyAlerterEndpoint {
    /** The ntfy topic URL */
    url: string;
    /**
     * Optional E-Mail Address to enable ntfy email notifications.
     * SMTP must be configured on the ntfy server.
     */
    email?: string;
}
/** Pauses all containers on the target server. Response: [Update] */
export interface PauseAllContainers {
    /** Name or id */
    server: string;
}
/**
 * Pauses the container on the target server. Response: [Update]
 *
 * 1. Runs `docker pause ${container_name}`.
 */
export interface PauseContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
}
/**
 * Pauses the container for the target deployment. Response: [Update]
 *
 * 1. Runs `docker pause ${container_name}`.
 */
export interface PauseDeployment {
    /** Name or id */
    deployment: string;
}
/** Pauses the target stack. `docker compose pause`. Response: [Update] */
export interface PauseStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only pause specific services.
     * If empty, will pause all services.
     */
    services?: string[];
}
export interface PermissionToml {
    /**
     * Id can be:
     * - resource name. `id = "abcd-build"`
     * - regex matching resource names. `id = "\^(.+)-build-([0-9]+)$\"`
     */
    target: ResourceTarget;
    /**
     * The permission level:
     * - None
     * - Read
     * - Execute
     * - Write
     */
    level?: PermissionLevel;
    /** Any [SpecificPermissions](SpecificPermission) on the resource */
    specific?: Array<SpecificPermission>;
}
/**
 * Prunes the docker buildx cache on the target server. Response: [Update].
 *
 * 1. Runs `docker buildx prune -a -f`.
 */
export interface PruneBuildx {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker containers on the target server. Response: [Update].
 *
 * 1. Runs `docker container prune -f`.
 */
export interface PruneContainers {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker builders (build cache) on the target server. Response: [Update].
 *
 * 1. Runs `docker builder prune -a -f`.
 */
export interface PruneDockerBuilders {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker images on the target server. Response: [Update].
 *
 * 1. Runs `docker image prune -a -f`.
 */
export interface PruneImages {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker networks on the target server. Response: [Update].
 *
 * 1. Runs `docker network prune -f`.
 */
export interface PruneNetworks {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker system on the target server, including volumes. Response: [Update].
 *
 * 1. Runs `docker system prune -a -f --volumes`.
 */
export interface PruneSystem {
    /** Id or name */
    server: string;
}
/**
 * Prunes the docker volumes on the target server. Response: [Update].
 *
 * 1. Runs `docker volume prune -a -f`.
 */
export interface PruneVolumes {
    /** Id or name */
    server: string;
}
/** Pulls the image for the target deployment. Response: [Update] */
export interface PullDeployment {
    /** Name or id */
    deployment: string;
}
/**
 * Pulls the target repo. Response: [Update].
 *
 * Note. Repo must have server attached at `server_id`.
 *
 * 1. Pulls the repo on the target server using `git pull`.
 * 2. If `on_pull` is specified, it will be executed after the pull is complete.
 */
export interface PullRepo {
    /** Id or name */
    repo: string;
}
/** Pulls images for the target stack. `docker compose pull`. Response: [Update] */
export interface PullStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only pull specific services.
     * If empty, will pull all services.
     */
    services?: string[];
}
/**
 * Push a resource to the front of the users 10 most recently viewed resources.
 * Response: [NoData].
 */
export interface PushRecentlyViewed {
    /** The target to push. */
    resource: ResourceTarget;
}
/** Configuration for a Pushover alerter. */
export interface PushoverAlerterEndpoint {
    /** The pushover URL including application and user tokens in parameters. */
    url: string;
}
/** Trigger a refresh of the cached latest hash and message. */
export interface RefreshBuildCache {
    /** Id or name */
    build: string;
}
/** Trigger a refresh of the cached latest hash and message. */
export interface RefreshRepoCache {
    /** Id or name */
    repo: string;
}
/** Trigger a refresh of the computed diff logs for view. Response: [ResourceSync] */
export interface RefreshResourceSyncPending {
    /** Id or name */
    sync: string;
}
/**
 * Trigger a refresh of the cached compose file contents.
 * Refreshes:
 * - Whether the remote file is missing
 * - The latest json, and for repos, the remote contents, hash, and message.
 */
export interface RefreshStackCache {
    /** Id or name */
    stack: string;
}
/**
 * `docker config rm CONFIG [CONFIG...]`
 *
 * https://docs.docker.com/reference/cli/docker/config/rm/
 */
export interface RemoveSwarmConfigs {
    /** Name or id */
    swarm: string;
    /** Config names or ids */
    configs: string[];
}
/**
 * `docker node rm [OPTIONS] NODE [NODE...]`
 *
 * https://docs.docker.com/reference/cli/docker/node/rm/
 */
export interface RemoveSwarmNodes {
    /** Name or id */
    swarm: string;
    /** Node names or ids to remove */
    nodes: string[];
    /** Force remove a node from the swarm */
    force?: boolean;
}
/**
 * `docker secret rm SECRET [SECRET...]`
 *
 * https://docs.docker.com/reference/cli/docker/secret/rm/
 */
export interface RemoveSwarmSecrets {
    /** Name or id */
    swarm: string;
    /** Secret names or ids */
    secrets: string[];
}
/**
 * `docker service rm SERVICE [SERVICE...]`
 *
 * https://docs.docker.com/reference/cli/docker/service/rm/
 */
export interface RemoveSwarmServices {
    /** Name or id */
    swarm: string;
    /** Service names or ids */
    services: string[];
}
/**
 * `docker stack rm [OPTIONS] STACK [STACK...]`
 *
 * https://docs.docker.com/reference/cli/docker/stack/rm/
 */
export interface RemoveSwarmStacks {
    /** Name or id */
    swarm: string;
    /** Node names to remove */
    stacks: string[];
    /** Do not wait for stack removal */
    detach: boolean;
}
/** **Admin only.** Remove a user from a user group. Response: [UserGroup] */
export interface RemoveUserFromUserGroup {
    /** The name or id of UserGroup that user should be removed from. */
    user_group: string;
    /** The id or username of the user to remove */
    user: string;
}
/**
 * Rename the Action at id to the given name.
 * Response: [Update].
 */
export interface RenameAction {
    /** The id or name of the Action to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Alerter at id to the given name.
 * Response: [Update].
 */
export interface RenameAlerter {
    /** The id or name of the Alerter to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Build at id to the given name.
 * Response: [Update].
 */
export interface RenameBuild {
    /** The id or name of the Build to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Builder at id to the given name.
 * Response: [Update].
 */
export interface RenameBuilder {
    /** The id or name of the Builder to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the deployment at id to the given name. Response: [Update].
 *
 * Note. If a container is created for the deployment, it will be renamed using
 * `docker rename ...`.
 */
export interface RenameDeployment {
    /** The id of the deployment to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Procedure at id to the given name.
 * Response: [Update].
 */
export interface RenameProcedure {
    /** The id or name of the Procedure to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Repo at id to the given name.
 * Response: [Update].
 */
export interface RenameRepo {
    /** The id or name of the Repo to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the ResourceSync at id to the given name.
 * Response: [Update].
 */
export interface RenameResourceSync {
    /** The id or name of the ResourceSync to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename an Server to the given name.
 * Response: [Update].
 */
export interface RenameServer {
    /** The id or name of the Server to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/** Rename the stack at id to the given name. Response: [Update]. */
export interface RenameStack {
    /** The id of the stack to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/**
 * Rename the Swarm at id to the given name.
 * Response: [Update].
 */
export interface RenameSwarm {
    /** The id or name of the Swarm to rename. */
    id: string;
    /** The new name. */
    name: string;
}
/** Rename a tag at id. Response: [Tag]. */
export interface RenameTag {
    /** The id of the tag to rename. */
    id: string;
    /** The new name of the tag. */
    name: string;
}
/** **Admin only.** Rename a user group. Response: [UserGroup] */
export interface RenameUserGroup {
    /** The id of the UserGroup */
    id: string;
    /** The new name for the UserGroup */
    name: string;
}
export declare enum DefaultRepoFolder {
    /** /${root_directory}/stacks */
    Stacks = "Stacks",
    /** /${root_directory}/builds */
    Builds = "Builds",
    /** /${root_directory}/repos */
    Repos = "Repos",
    /**
     * If the repo is only cloned
     * in the core repo cache (resource sync),
     * this isn't relevant.
     */
    NotApplicable = "NotApplicable"
}
export interface RepoExecutionArgs {
    /** Resource name (eg Build name, Repo name) */
    name: string;
    /** Git provider domain. Default: `github.com` */
    provider: string;
    /** Use https (vs http). */
    https: boolean;
    /** Configure the account used to access repo (if private) */
    account?: string;
    /**
     * Full repo identifier. {namespace}/{repo_name}
     * Its optional to force checking and produce error if not defined.
     */
    repo?: string;
    /** Git Branch. Default: `main` */
    branch: string;
    /** Specific commit hash. Optional */
    commit?: string;
    /** The clone destination path */
    destination?: string;
    /**
     * The default folder to use.
     * Depends on the resource type.
     */
    default_folder: DefaultRepoFolder;
}
export interface RepoExecutionResponse {
    /** Response logs */
    logs: Log[];
    /** Absolute path to the repo root on the host. */
    path: string;
    /** Latest short commit hash, if it could be retrieved */
    commit_hash?: string;
    /** Latest commit message, if it could be retrieved */
    commit_message?: string;
}
export interface ResourceToml<PartialConfig> {
    /** The resource name. Required */
    name: string;
    /** The resource description. Optional. */
    description?: string;
    /** Mark resource as a template */
    template?: boolean;
    /** Tag ids or names. Optional */
    tags?: string[];
    /**
     * Optional. Only relevant for deployments / stacks.
     *
     * Will ensure deployment / stack is running with the latest configuration.
     * Deploy actions to achieve this will be included in the sync.
     * Default is false.
     */
    deploy?: boolean;
    /**
     * Optional. Only relevant for deployments / stacks using the 'deploy' sync feature.
     *
     * Specify other deployments / stacks by name as dependencies.
     * The sync will ensure the deployment / stack will only be deployed 'after' its dependencies.
     */
    after?: string[];
    /** Resource specific configuration. */
    config?: PartialConfig;
}
export interface UserGroupToml {
    /** User group name */
    name: string;
    /** Whether all users will implicitly have the permissions in this group. */
    everyone?: boolean;
    /** Users in the group */
    users?: string[];
    /** Give the user group elevated permissions on all resources of a certain type */
    all?: Record<ResourceTarget["type"], PermissionLevelAndSpecifics | PermissionLevel>;
    /** Permissions given to the group */
    permissions?: PermissionToml[];
}
/** Specifies resources to sync on Komodo */
export interface ResourcesToml {
    swarms?: ResourceToml<_PartialSwarmConfig>[];
    servers?: ResourceToml<_PartialServerConfig>[];
    deployments?: ResourceToml<_PartialDeploymentConfig>[];
    stacks?: ResourceToml<_PartialStackConfig>[];
    builds?: ResourceToml<_PartialBuildConfig>[];
    repos?: ResourceToml<_PartialRepoConfig>[];
    procedures?: ResourceToml<_PartialProcedureConfig>[];
    actions?: ResourceToml<_PartialActionConfig>[];
    alerters?: ResourceToml<_PartialAlerterConfig>[];
    builders?: ResourceToml<_PartialBuilderConfig>[];
    resource_syncs?: ResourceToml<_PartialResourceSyncConfig>[];
    user_groups?: UserGroupToml[];
    variables?: Variable[];
}
/** Restarts all containers on the target server. Response: [Update] */
export interface RestartAllContainers {
    /** Name or id */
    server: string;
}
/**
 * Restarts the container on the target server. Response: [Update]
 *
 * 1. Runs `docker restart ${container_name}`.
 */
export interface RestartContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
}
/**
 * Restarts the container for the target deployment. Response: [Update]
 *
 * 1. Runs `docker restart ${container_name}`.
 */
export interface RestartDeployment {
    /** Name or id */
    deployment: string;
}
/** Restarts the target stack. `docker compose restart`. Response: [Update] */
export interface RestartStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only restart specific services.
     * If empty, will restart all services.
     */
    services?: string[];
}
/**
 * **Admin only.** Rotates all connected Server keys.
 * Response: [Update]. Alias: `rotate-keys`.
 */
export interface RotateAllServerKeys {
}
/**
 * **Admin only.** Rotates the Core private key,
 * and all Server public keys.
 * Response: [Update].
 *
 * If any Server is `NotOk`, this will fail.
 * To proceed anyways, pass `force: true`.
 */
export interface RotateCoreKeys {
    /**
     * Force the rotation to proceed even if a Server is `NotOk`.
     * The Core Public Key in Periphery config may have to be updated manually.
     * (alias: `f`)
     */
    force?: boolean;
}
/**
 * Rotates the private / public keys for the server.
 * Response: [Update]
 */
export interface RotateServerKeys {
    /** Server Id or name */
    server: string;
}
/** Runs the target Action. Response: [Update] */
export interface RunAction {
    /** Id or name */
    action: string;
    /**
     * Custom arguments which are merged on top of the default arguments.
     * CLI Format: `"VAR1=val1&VAR2=val2"`
     *
     * Webhook-triggered actions use this to pass WEBHOOK_BRANCH and WEBHOOK_BODY.
     */
    args?: JsonObject;
}
/**
 * Runs the target build. Response: [Update].
 *
 * 1. Get a handle to the builder. If using AWS builder, this means starting a builder ec2 instance.
 *
 * 2. Clone the repo on the builder. If an `on_clone` commmand is given, it will be executed.
 *
 * 3. Execute `docker build {...params}`, where params are determined using the builds configuration.
 *
 * 4. If a docker registry is configured, the build will be pushed to the registry.
 *
 * 5. If using AWS builder, destroy the builder ec2 instance.
 *
 * 6. Deploy any Deployments with *Redeploy on Build* enabled.
 */
export interface RunBuild {
    /** Can be build id or name */
    build: string;
}
/** Runs the target Procedure. Response: [Update] */
export interface RunProcedure {
    /** Id or name */
    procedure: string;
}
/** Runs a one-time command against a service using `docker compose run`. Response: [Update] */
export interface RunStackService {
    /** Id or name */
    stack: string;
    /** Service to run */
    service: string;
    /** Command and args to pass to the service container */
    command?: string[];
    /** Do not allocate TTY */
    no_tty?: boolean;
    /** Do not start linked services */
    no_deps?: boolean;
    /** Detach container on run */
    detach?: boolean;
    /** Map service ports to the host */
    service_ports?: boolean;
    /** Extra environment variables for the run */
    env?: Record<string, string>;
    /** Working directory inside the container */
    workdir?: string;
    /** User to run as inside the container */
    user?: string;
    /** Override the default entrypoint */
    entrypoint?: string;
    /** Pull the image before running */
    pull?: boolean;
}
/** Runs the target resource sync. Response: [Update] */
export interface RunSync {
    /** Id or name */
    sync: string;
    /**
     * Only execute sync on a specific resource type.
     * Combine with `resource_id` to specify resource.
     */
    resource_type?: ResourceTarget["type"];
    /**
     * Only execute sync on a specific resources.
     * Combine with `resource_type` to specify resources.
     * Supports name or id.
     */
    resources?: string[];
}
export declare enum SearchCombinator {
    Or = "Or",
    And = "And"
}
/**
 * Search the container log's tail using `grep`. All lines go to stdout.
 * Response: [Log].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface SearchContainerLog {
    /** Id or name */
    server: string;
    /** The container name */
    container: string;
    /** The terms to search for. */
    terms: string[];
    /**
     * When searching for multiple terms, can use `AND` or `OR` combinator.
     *
     * - `AND`: Only include lines with **all** terms present in that line.
     * - `OR`: Include lines that have one or more matches in the terms.
     */
    combinator?: SearchCombinator;
    /** Invert the results, ie return all lines that DON'T match the terms / combinator. */
    invert?: boolean;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Search the deployment log's tail using `grep`. All lines go to stdout.
 * Response: [Log].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface SearchDeploymentLog {
    /** Id or name */
    deployment: string;
    /** The terms to search for. */
    terms: string[];
    /**
     * When searching for multiple terms, can use `AND` or `OR` combinator.
     *
     * - `AND`: Only include lines with **all** terms present in that line.
     * - `OR`: Include lines that have one or more matches in the terms.
     */
    combinator?: SearchCombinator;
    /** Invert the results, ie return all lines that DON'T match the terms / combinator. */
    invert?: boolean;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Search the stack log's tail using `grep`. All lines go to stdout.
 * Response: [SearchStackLogResponse].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface SearchStackLog {
    /** Id or name */
    stack: string;
    /**
     * Filter the logs to only ones from specific services.
     * If empty, will include logs from all services.
     */
    services: string[];
    /** The terms to search for. */
    terms: string[];
    /**
     * When searching for multiple terms, can use `AND` or `OR` combinator.
     *
     * - `AND`: Only include lines with **all** terms present in that line.
     * - `OR`: Include lines that have one or more matches in the terms.
     */
    combinator?: SearchCombinator;
    /** Invert the results, ie return all lines that DON'T match the terms / combinator. */
    invert?: boolean;
    /** Enable `--timestamps` */
    timestamps?: boolean;
}
/**
 * Search the swarm service log's tail using `grep`. All lines go to stdout.
 * Response: [SearchSwarmServiceLogResponse].
 *
 * Note. This call will hit the underlying server directly for most up to date log.
 */
export interface SearchSwarmServiceLog {
    /** Id or name */
    swarm: string;
    /** Select the swarm service to get logs for. */
    service: string;
    /** The terms to search for. */
    terms: string[];
    /**
     * When searching for multiple terms, can use `AND` or `OR` combinator.
     *
     * - `AND`: Only include lines with **all** terms present in that line.
     * - `OR`: Include lines that have one or more matches in the terms.
     */
    combinator?: SearchCombinator;
    /** Invert the results, ie return all lines that DON'T match the terms / combinator. */
    invert?: boolean;
    /** Enable `--timestamps` */
    timestamps?: boolean;
    /** Enable `--no-task-ids` */
    no_task_ids?: boolean;
    /** Enable `--no-resolve` */
    no_resolve?: boolean;
    /** Enable `--details` */
    details?: boolean;
}
/**
 * Send a custom alert message to configured Alerters. Response: [Update].
 * Alias: `alert`
 */
export interface SendAlert {
    /** The alert level. */
    level?: SeverityLevel;
    /** The alert message. Required. */
    message: string;
    /** The alert details. Optional. */
    details?: string;
    /**
     * Specific alerter names or ids.
     * If empty / not passed, sends to all configured alerters
     * with the `Custom` alert type whitelisted / not blacklisted.
     */
    alerters?: string[];
}
/** Configuration for a Komodo Server Builder. */
export interface ServerBuilderConfig {
    /** The server id of the builder */
    server_id?: string;
}
/** The health of a part of the server. */
export interface ServerHealthState {
    level: SeverityLevel;
    /** Whether the health is good enough to close an open alert. */
    should_close_alert: boolean;
}
/** Summary of the health of the server. */
export interface ServerHealth {
    cpu: ServerHealthState;
    mem: ServerHealthState;
    disks: Record<string, ServerHealthState>;
}
/**
 * **Admin only.** Set `everyone` property of User Group.
 * Response: [UserGroup]
 */
export interface SetEveryoneUserGroup {
    /** Id or name. */
    user_group: string;
    /** Whether this user group applies to everyone. */
    everyone: boolean;
}
/**
 * Set the time the user last opened the UI updates.
 * Used for unseen notification dot.
 * Response: [NoData]
 */
export interface SetLastSeenUpdate {
}
/**
 * **Admin only.** Completely override the users in the group.
 * Response: [UserGroup]
 */
export interface SetUsersInUserGroup {
    /** Id or name. */
    user_group: string;
    /** The user ids or usernames to hard set as the group's users. */
    users: string[];
}
/**
 * Sign up a new local user account. Will fail if a user with the
 * given username already exists.
 * Response: [SignUpLocalUserResponse].
 *
 * Note. This method is only available if the core api has `local_auth` enabled,
 * and if user registration is not disabled (after the first user).
 */
export interface SignUpLocalUser {
    /** The username for the new user. */
    username: string;
    /**
     * The password for the new user.
     * This cannot be retreived later.
     */
    password: string;
}
/** Info for network interface usage. */
export interface SingleNetworkInterfaceUsage {
    /** The network interface name */
    name: string;
    /** The ingress in bytes */
    ingress_bytes: number;
    /** The egress in bytes */
    egress_bytes: number;
}
/** Configuration for a Slack alerter. */
export interface SlackAlerterEndpoint {
    /** The Slack app webhook url */
    url: string;
}
/** Sleeps for the specified time. */
export interface Sleep {
    duration_ms?: I64;
}
/** Starts all containers on the target server. Response: [Update] */
export interface StartAllContainers {
    /** Name or id */
    server: string;
}
/**
 * Starts the container on the target server. Response: [Update]
 *
 * 1. Runs `docker start ${container_name}`.
 */
export interface StartContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
}
/**
 * Starts the container for the target deployment. Response: [Update]
 *
 * 1. Runs `docker start ${container_name}`.
 */
export interface StartDeployment {
    /** Name or id */
    deployment: string;
}
/** Starts the target stack. `docker compose start`. Response: [Update] */
export interface StartStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only start specific services.
     * If empty, will start all services.
     */
    services?: string[];
}
/** Stops all containers on the target server. Response: [Update] */
export interface StopAllContainers {
    /** Name or id */
    server: string;
}
/**
 * Stops the container on the target server. Response: [Update]
 *
 * 1. Runs `docker stop ${container_name}`.
 */
export interface StopContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
    /** Override the default termination signal. */
    signal?: TerminationSignal;
    /** Override the default termination max time. */
    time?: number;
}
/**
 * Stops the container for the target deployment. Response: [Update]
 *
 * 1. Runs `docker stop ${container_name}`.
 */
export interface StopDeployment {
    /** Name or id */
    deployment: string;
    /** Override the default termination signal specified in the deployment. */
    signal?: TerminationSignal;
    /** Override the default termination max time. */
    time?: number;
}
/** Stops the target stack. `docker compose stop`. Response: [Update] */
export interface StopStack {
    /** Id or name */
    stack: string;
    /** Override the default termination max time. */
    stop_time?: number;
    /**
     * Filter to only stop specific services.
     * If empty, will stop all services.
     */
    services?: string[];
}
/** Swarm config details. */
export interface SwarmConfig {
    ID?: string;
    Version?: ObjectVersion;
    CreatedAt?: string;
    UpdatedAt?: string;
    Spec?: ConfigSpec;
}
/** JSON structure to send new terminal window dimensions */
export interface TerminalResizeMessage {
    rows: number;
    cols: number;
}
export interface TerminationSignalLabel {
    signal: TerminationSignal;
    label: string;
}
/** Tests an Alerters ability to reach the configured endpoint. Response: [Update] */
export interface TestAlerter {
    /** Name or id */
    alerter: string;
}
/** Info for the all system disks combined. */
export interface TotalDiskUsage {
    /** Used portion in GB */
    used_gb: number;
    /** Total size in GB */
    total_gb: number;
}
/** Unpauses all containers on the target server. Response: [Update] */
export interface UnpauseAllContainers {
    /** Name or id */
    server: string;
}
/**
 * Unpauses the container on the target server. Response: [Update]
 *
 * 1. Runs `docker unpause ${container_name}`.
 *
 * Note. This is the only way to restart a paused container.
 */
export interface UnpauseContainer {
    /** Name or id */
    server: string;
    /** The container name */
    container: string;
}
/**
 * Unpauses the container for the target deployment. Response: [Update]
 *
 * 1. Runs `docker unpause ${container_name}`.
 *
 * Note. This is the only way to restart a paused container.
 */
export interface UnpauseDeployment {
    /** Name or id */
    deployment: string;
}
/**
 * Unpauses the target stack. `docker compose unpause`. Response: [Update].
 *
 * Note. This is the only way to restart a paused container.
 */
export interface UnpauseStack {
    /** Id or name */
    stack: string;
    /**
     * Filter to only unpause specific services.
     * If empty, will unpause all services.
     */
    services?: string[];
}
/**
 * Update the action at the given id, and return the updated action.
 * Response: [Action].
 *
 * Note. This method updates only the fields which are set in the [_PartialActionConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateAction {
    /** The id of the action to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialActionConfig;
}
/**
 * Update the alerter at the given id, and return the updated alerter. Response: [Alerter].
 *
 * Note. This method updates only the fields which are set in the [PartialAlerterConfig][crate::entities::alerter::PartialAlerterConfig],
 * effectively merging diffs into the final document. This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateAlerter {
    /** The id of the alerter to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialAlerterConfig;
}
/**
 * Update the build at the given id, and return the updated build.
 * Response: [Build].
 *
 * Note. This method updates only the fields which are set in the [_PartialBuildConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateBuild {
    /** The id or name of the build to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialBuildConfig;
}
/**
 * Update the builder at the given id, and return the updated builder.
 * Response: [Builder].
 *
 * Note. This method updates only the fields which are set in the [PartialBuilderConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateBuilder {
    /** The id of the builder to update. */
    id: string;
    /** The partial config update to apply. */
    config: PartialBuilderConfig;
}
/**
 * Update the deployment at the given id, and return the updated deployment.
 * Response: [Deployment].
 *
 * Note. If the attached server for the deployment changes,
 * the deployment will be deleted / cleaned up on the old server.
 *
 * Note. This method updates only the fields which are set in the [_PartialDeploymentConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateDeployment {
    /** The deployment id to update. */
    id: string;
    /** The partial config update. */
    config: _PartialDeploymentConfig;
}
/**
 * **Admin only.** Update a docker registry account.
 * Response: [DockerRegistryAccount].
 */
export interface UpdateDockerRegistryAccount {
    /** The id of the docker registry to update */
    id: string;
    /** The partial docker registry account. */
    account: _PartialDockerRegistryAccount;
}
/**
 * **Admin only.** Update a git provider account.
 * Response: [GitProviderAccount].
 */
export interface UpdateGitProviderAccount {
    /** The id of the git provider account to update. */
    id: string;
    /** The partial git provider account. */
    account: _PartialGitProviderAccount;
}
/**
 * **Admin only.** Delete an onboarding key.
 * Response: The deleted [OnboardingKey].
 */
export interface UpdateOnboardingKey {
    /** The onboarding public key. */
    public_key: string;
    /** Update the key enabled state. */
    enabled?: boolean;
    /** Update the key name */
    name?: string;
    /** Update the onboarding key expire time. */
    expires?: I64;
    /** Update the tags */
    tags?: string[];
    /** Update the copy server */
    copy_server?: string;
    /** Update whether to create Builder */
    create_builder?: boolean;
}
/**
 * **Admin only.** Update a user or user groups base permission level on a resource type.
 * Response: [NoData].
 */
export interface UpdatePermissionOnResourceType {
    /** Specify the user or user group. */
    user_target: UserTarget;
    /** The resource type: eg. Server, Build, Deployment, etc. */
    resource_type: ResourceTarget["type"];
    /** The base permission level. */
    permission: PermissionLevelAndSpecifics | PermissionLevel;
}
/**
 * **Admin only.** Update a user or user groups permission on a resource.
 * Response: [NoData].
 */
export interface UpdatePermissionOnTarget {
    /** Specify the user or user group. */
    user_target: UserTarget;
    /** Specify the target resource. */
    resource_target: ResourceTarget;
    /** Specify the permission level. */
    permission: PermissionLevelAndSpecifics | PermissionLevel;
}
/**
 * Update the procedure at the given id, and return the updated procedure.
 * Response: [Procedure].
 *
 * Note. This method updates only the fields which are set in the [_PartialProcedureConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateProcedure {
    /** The id of the procedure to update. */
    id: string;
    /** The partial config update. */
    config: _PartialProcedureConfig;
}
/**
 * Update the repo at the given id, and return the updated repo.
 * Response: [Repo].
 *
 * Note. If the attached server for the repo changes,
 * the repo will be deleted / cleaned up on the old server.
 *
 * Note. This method updates only the fields which are set in the [_PartialRepoConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateRepo {
    /** The id of the repo to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialRepoConfig;
}
/**
 * Update a resources common meta fields.
 * - description
 * - template
 * - tags
 * Response: [NoData].
 */
export interface UpdateResourceMeta {
    /** The target resource to set update meta. */
    target: ResourceTarget;
    /**
     * New description to set,
     * or null for no update
     */
    description?: string;
    /**
     * New template value (true or false),
     * or null for no update
     */
    template?: boolean;
    /**
     * The exact tags to set,
     * or null for no update
     */
    tags?: string[];
}
/**
 * Update the sync at the given id, and return the updated sync.
 * Response: [ResourceSync].
 *
 * Note. This method updates only the fields which are set in the [_PartialResourceSyncConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateResourceSync {
    /** The id of the sync to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialResourceSyncConfig;
}
/**
 * Update the server at the given id, and return the updated server.
 * Response: [Server].
 *
 * Note. This method updates only the fields which are set in the [_PartialServerConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateServer {
    /** The id or name of the server to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialServerConfig;
}
/**
 * Updates the Server with an explicit Public Key.
 * Response: [Update]
 */
export interface UpdateServerPublicKey {
    /** Server Id or name */
    server: string;
    /** Spki base64 public key */
    public_key: string;
}
/**
 * **Admin only.** Update a service user's description.
 * Response: [User].
 */
export interface UpdateServiceUserDescription {
    /** The service user's username */
    username: string;
    /** A new description for the service user. */
    description: string;
}
/**
 * Update the stack at the given id, and return the updated stack.
 * Response: [Stack].
 *
 * Note. If the attached server for the stack changes,
 * the stack will be deleted / cleaned up on the old server.
 *
 * Note. This method updates only the fields which are set in the [_PartialStackConfig],
 * merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateStack {
    /** The id of the Stack to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialStackConfig;
}
/**
 * Update the Swarm at the given id, and return the updated Swarm.
 * Response: [Swarm].
 *
 * Note. If the attached server for the Swarm changes,
 * the Swarm will be deleted / cleaned up on the old server.
 *
 * Note. This method updates only the fields which are set in the [_PartialSwarmConfig],
 * effectively merging diffs into the final document.
 * This is helpful when multiple users are using
 * the same resources concurrently by ensuring no unintentional
 * field changes occur from out of date local state.
 */
export interface UpdateSwarm {
    /** The id of the swarm to update. */
    id: string;
    /** The partial config update to apply. */
    config: _PartialSwarmConfig;
}
/** Update color for tag. Response: [Tag]. */
export interface UpdateTagColor {
    /** The name or id of the tag to update. */
    tag: string;
    /** The new color for the tag. */
    color: TagColor;
}
/**
 * **Super Admin only.** Update's whether a user is admin.
 * Response: [NoData].
 */
export interface UpdateUserAdmin {
    /** The target user. */
    user_id: string;
    /** Whether user should be admin. */
    admin: boolean;
}
/**
 * **Admin only.** Update a user's "base" permissions, eg. "enabled".
 * Response: [NoData].
 */
export interface UpdateUserBasePermissions {
    /** The target user. */
    user_id: string;
    /** If specified, will update users enabled state. */
    enabled?: boolean;
    /** If specified, will update user's ability to create servers. */
    create_servers?: boolean;
    /** If specified, will update user's ability to create builds. */
    create_builds?: boolean;
}
/**
 * **Only for local users**. Update the calling users password.
 * Response: [NoData].
 */
export interface UpdateUserPassword {
    password: string;
}
/**
 * **Only for local users**. Update the calling users username.
 * Response: [NoData].
 */
export interface UpdateUserUsername {
    username: string;
}
/** **Admin only.** Update variable description. Response: [Variable]. */
export interface UpdateVariableDescription {
    /** The name of the variable to update. */
    name: string;
    /** The description to set. */
    description: string;
}
/** **Admin only.** Update whether variable is secret. Response: [Variable]. */
export interface UpdateVariableIsSecret {
    /** The name of the variable to update. */
    name: string;
    /** Whether variable is secret. */
    is_secret: boolean;
}
/** **Admin only.** Update variable value. Response: [Variable]. */
export interface UpdateVariableValue {
    /** The name of the variable to update. */
    name: string;
    /** The value to set. */
    value: string;
}
/** Configuration for a Komodo Url Builder. */
export interface UrlBuilderConfig {
    /** The address of the Periphery agent */
    address: string;
    /**
     * An expected public key associated with Periphery private key.
     * If empty, doesn't validate Periphery public key.
     */
    periphery_public_key?: string;
    /** Whether to validate the Periphery tls certificates. */
    insecure_tls: boolean;
    /**
     * Deprecated. Use private / public keys instead.
     * An optional override passkey to use
     * to authenticate with periphery agent.
     * If this is empty, will use passkey in core config.
     */
    passkey?: string;
}
/** Update dockerfile contents in Files on Server or Git Repo mode. Response: [Update]. */
export interface WriteBuildFileContents {
    /** The name or id of the target Build. */
    build: string;
    /** The dockerfile contents to write. */
    contents: string;
}
/** Update file contents in Files on Server or Git Repo mode. Response: [Update]. */
export interface WriteStackFileContents {
    /** The name or id of the target Stack. */
    stack: string;
    /**
     * The file path relative to the stack run directory,
     * or absolute path.
     */
    file_path: string;
    /** The contents to write. */
    contents: string;
}
/** Rename the stack at id to the given name. Response: [Update]. */
export interface WriteSyncFileContents {
    /** The name or id of the target Sync. */
    sync: string;
    /**
     * If this file was under a resource folder, this will be the folder.
     * Otherwise, it should be empty string.
     */
    resource_path: string;
    /** The file path relative to the resource path. */
    file_path: string;
    /** The contents to write. */
    contents: string;
}
export type AuthRequest = {
    type: "GetLoginOptions";
    params: GetLoginOptions;
} | {
    type: "SignUpLocalUser";
    params: SignUpLocalUser;
} | {
    type: "LoginLocalUser";
    params: LoginLocalUser;
} | {
    type: "ExchangeForJwt";
    params: ExchangeForJwt;
} | {
    type: "GetUser";
    params: GetUser;
};
/** Days of the week */
export declare enum DayOfWeek {
    Monday = "Monday",
    Tuesday = "Tuesday",
    Wednesday = "Wednesday",
    Thursday = "Thursday",
    Friday = "Friday",
    Saturday = "Saturday",
    Sunday = "Sunday"
}
export type ExecuteRequest = {
    type: "DeployStack";
    params: DeployStack;
} | {
    type: "BatchDeployStack";
    params: BatchDeployStack;
} | {
    type: "DeployStackIfChanged";
    params: DeployStackIfChanged;
} | {
    type: "BatchDeployStackIfChanged";
    params: BatchDeployStackIfChanged;
} | {
    type: "PullStack";
    params: PullStack;
} | {
    type: "BatchPullStack";
    params: BatchPullStack;
} | {
    type: "StartStack";
    params: StartStack;
} | {
    type: "RestartStack";
    params: RestartStack;
} | {
    type: "StopStack";
    params: StopStack;
} | {
    type: "PauseStack";
    params: PauseStack;
} | {
    type: "UnpauseStack";
    params: UnpauseStack;
} | {
    type: "DestroyStack";
    params: DestroyStack;
} | {
    type: "BatchDestroyStack";
    params: BatchDestroyStack;
} | {
    type: "RunStackService";
    params: RunStackService;
} | {
    type: "Deploy";
    params: Deploy;
} | {
    type: "BatchDeploy";
    params: BatchDeploy;
} | {
    type: "PullDeployment";
    params: PullDeployment;
} | {
    type: "StartDeployment";
    params: StartDeployment;
} | {
    type: "RestartDeployment";
    params: RestartDeployment;
} | {
    type: "PauseDeployment";
    params: PauseDeployment;
} | {
    type: "UnpauseDeployment";
    params: UnpauseDeployment;
} | {
    type: "StopDeployment";
    params: StopDeployment;
} | {
    type: "DestroyDeployment";
    params: DestroyDeployment;
} | {
    type: "BatchDestroyDeployment";
    params: BatchDestroyDeployment;
} | {
    type: "RunBuild";
    params: RunBuild;
} | {
    type: "BatchRunBuild";
    params: BatchRunBuild;
} | {
    type: "CancelBuild";
    params: CancelBuild;
} | {
    type: "CloneRepo";
    params: CloneRepo;
} | {
    type: "BatchCloneRepo";
    params: BatchCloneRepo;
} | {
    type: "PullRepo";
    params: PullRepo;
} | {
    type: "BatchPullRepo";
    params: BatchPullRepo;
} | {
    type: "BuildRepo";
    params: BuildRepo;
} | {
    type: "BatchBuildRepo";
    params: BatchBuildRepo;
} | {
    type: "CancelRepoBuild";
    params: CancelRepoBuild;
} | {
    type: "RunProcedure";
    params: RunProcedure;
} | {
    type: "BatchRunProcedure";
    params: BatchRunProcedure;
} | {
    type: "RunAction";
    params: RunAction;
} | {
    type: "BatchRunAction";
    params: BatchRunAction;
} | {
    type: "RunSync";
    params: RunSync;
} | {
    type: "TestAlerter";
    params: TestAlerter;
} | {
    type: "SendAlert";
    params: SendAlert;
} | {
    type: "StartContainer";
    params: StartContainer;
} | {
    type: "RestartContainer";
    params: RestartContainer;
} | {
    type: "PauseContainer";
    params: PauseContainer;
} | {
    type: "UnpauseContainer";
    params: UnpauseContainer;
} | {
    type: "StopContainer";
    params: StopContainer;
} | {
    type: "DestroyContainer";
    params: DestroyContainer;
} | {
    type: "StartAllContainers";
    params: StartAllContainers;
} | {
    type: "RestartAllContainers";
    params: RestartAllContainers;
} | {
    type: "PauseAllContainers";
    params: PauseAllContainers;
} | {
    type: "UnpauseAllContainers";
    params: UnpauseAllContainers;
} | {
    type: "StopAllContainers";
    params: StopAllContainers;
} | {
    type: "PruneContainers";
    params: PruneContainers;
} | {
    type: "DeleteNetwork";
    params: DeleteNetwork;
} | {
    type: "PruneNetworks";
    params: PruneNetworks;
} | {
    type: "DeleteImage";
    params: DeleteImage;
} | {
    type: "PruneImages";
    params: PruneImages;
} | {
    type: "DeleteVolume";
    params: DeleteVolume;
} | {
    type: "PruneVolumes";
    params: PruneVolumes;
} | {
    type: "PruneDockerBuilders";
    params: PruneDockerBuilders;
} | {
    type: "PruneBuildx";
    params: PruneBuildx;
} | {
    type: "PruneSystem";
    params: PruneSystem;
} | {
    type: "RemoveSwarmNodes";
    params: RemoveSwarmNodes;
} | {
    type: "RemoveSwarmStacks";
    params: RemoveSwarmStacks;
} | {
    type: "RemoveSwarmServices";
    params: RemoveSwarmServices;
} | {
    type: "RemoveSwarmConfigs";
    params: RemoveSwarmConfigs;
} | {
    type: "RemoveSwarmSecrets";
    params: RemoveSwarmSecrets;
} | {
    type: "ClearRepoCache";
    params: ClearRepoCache;
} | {
    type: "BackupCoreDatabase";
    params: BackupCoreDatabase;
} | {
    type: "GlobalAutoUpdate";
    params: GlobalAutoUpdate;
} | {
    type: "RotateAllServerKeys";
    params: RotateAllServerKeys;
} | {
    type: "RotateCoreKeys";
    params: RotateCoreKeys;
};
/**
 * One representative IANA zone for each distinct base UTC offset in the tz database.
 * https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
 *
 * The `serde`/`strum` renames ensure the canonical identifier is used
 * when serializing or parsing from a string such as `"Etc/UTC"`.
 */
export declare enum IanaTimezone {
    /** UTC−12:00 */
    EtcGmtMinus12 = "Etc/GMT+12",
    /** UTC−11:00 */
    PacificPagoPago = "Pacific/Pago_Pago",
    /** UTC−10:00 */
    PacificHonolulu = "Pacific/Honolulu",
    /** UTC−09:30 */
    PacificMarquesas = "Pacific/Marquesas",
    /** UTC−09:00 */
    AmericaAnchorage = "America/Anchorage",
    /** UTC−08:00 */
    AmericaLosAngeles = "America/Los_Angeles",
    /** UTC−07:00 */
    AmericaDenver = "America/Denver",
    /** UTC−06:00 */
    AmericaChicago = "America/Chicago",
    /** UTC−05:00 */
    AmericaNewYork = "America/New_York",
    /** UTC−04:00 */
    AmericaHalifax = "America/Halifax",
    /** UTC−03:30 */
    AmericaStJohns = "America/St_Johns",
    /** UTC−03:00 */
    AmericaSaoPaulo = "America/Sao_Paulo",
    /** UTC−02:00 */
    AmericaNoronha = "America/Noronha",
    /** UTC−01:00 */
    AtlanticAzores = "Atlantic/Azores",
    /** UTC±00:00 */
    EtcUtc = "Etc/UTC",
    /** UTC+01:00 */
    EuropeBerlin = "Europe/Berlin",
    /** UTC+02:00 */
    EuropeBucharest = "Europe/Bucharest",
    /** UTC+03:00 */
    EuropeMoscow = "Europe/Moscow",
    /** UTC+03:30 */
    AsiaTehran = "Asia/Tehran",
    /** UTC+04:00 */
    AsiaDubai = "Asia/Dubai",
    /** UTC+04:30 */
    AsiaKabul = "Asia/Kabul",
    /** UTC+05:00 */
    AsiaKarachi = "Asia/Karachi",
    /** UTC+05:30 */
    AsiaKolkata = "Asia/Kolkata",
    /** UTC+05:45 */
    AsiaKathmandu = "Asia/Kathmandu",
    /** UTC+06:00 */
    AsiaDhaka = "Asia/Dhaka",
    /** UTC+06:30 */
    AsiaYangon = "Asia/Yangon",
    /** UTC+07:00 */
    AsiaBangkok = "Asia/Bangkok",
    /** UTC+08:00 */
    AsiaShanghai = "Asia/Shanghai",
    /** UTC+08:45 */
    AustraliaEucla = "Australia/Eucla",
    /** UTC+09:00 */
    AsiaTokyo = "Asia/Tokyo",
    /** UTC+09:30 */
    AustraliaAdelaide = "Australia/Adelaide",
    /** UTC+10:00 */
    AustraliaSydney = "Australia/Sydney",
    /** UTC+10:30 */
    AustraliaLordHowe = "Australia/Lord_Howe",
    /** UTC+11:00 */
    PacificPortMoresby = "Pacific/Port_Moresby",
    /** UTC+12:00 */
    PacificAuckland = "Pacific/Auckland",
    /** UTC+12:45 */
    PacificChatham = "Pacific/Chatham",
    /** UTC+13:00 */
    PacificTongatapu = "Pacific/Tongatapu",
    /** UTC+14:00 */
    PacificKiritimati = "Pacific/Kiritimati"
}
export type ReadRequest = {
    type: "GetVersion";
    params: GetVersion;
} | {
    type: "GetCoreInfo";
    params: GetCoreInfo;
} | {
    type: "ListSecrets";
    params: ListSecrets;
} | {
    type: "ListGitProvidersFromConfig";
    params: ListGitProvidersFromConfig;
} | {
    type: "ListDockerRegistriesFromConfig";
    params: ListDockerRegistriesFromConfig;
} | {
    type: "GetSwarmsSummary";
    params: GetSwarmsSummary;
} | {
    type: "GetSwarm";
    params: GetSwarm;
} | {
    type: "GetSwarmActionState";
    params: GetSwarmActionState;
} | {
    type: "ListSwarms";
    params: ListSwarms;
} | {
    type: "InspectSwarm";
    params: InspectSwarm;
} | {
    type: "ListFullSwarms";
    params: ListFullSwarms;
} | {
    type: "ListSwarmNodes";
    params: ListSwarmNodes;
} | {
    type: "InspectSwarmNode";
    params: InspectSwarmNode;
} | {
    type: "ListSwarmConfigs";
    params: ListSwarmConfigs;
} | {
    type: "InspectSwarmConfig";
    params: InspectSwarmConfig;
} | {
    type: "ListSwarmSecrets";
    params: ListSwarmSecrets;
} | {
    type: "InspectSwarmSecret";
    params: InspectSwarmSecret;
} | {
    type: "ListSwarmStacks";
    params: ListSwarmStacks;
} | {
    type: "InspectSwarmStack";
    params: InspectSwarmStack;
} | {
    type: "ListSwarmTasks";
    params: ListSwarmTasks;
} | {
    type: "InspectSwarmTask";
    params: InspectSwarmTask;
} | {
    type: "ListSwarmServices";
    params: ListSwarmServices;
} | {
    type: "InspectSwarmService";
    params: InspectSwarmService;
} | {
    type: "GetSwarmServiceLog";
    params: GetSwarmServiceLog;
} | {
    type: "SearchSwarmServiceLog";
    params: SearchSwarmServiceLog;
} | {
    type: "GetServersSummary";
    params: GetServersSummary;
} | {
    type: "GetServer";
    params: GetServer;
} | {
    type: "GetServerState";
    params: GetServerState;
} | {
    type: "GetPeripheryInformation";
    params: GetPeripheryInformation;
} | {
    type: "GetServerActionState";
    params: GetServerActionState;
} | {
    type: "ListServers";
    params: ListServers;
} | {
    type: "ListFullServers";
    params: ListFullServers;
} | {
    type: "ListTerminals";
    params: ListTerminals;
} | {
    type: "GetDockerContainersSummary";
    params: GetDockerContainersSummary;
} | {
    type: "ListAllDockerContainers";
    params: ListAllDockerContainers;
} | {
    type: "ListDockerContainers";
    params: ListDockerContainers;
} | {
    type: "InspectDockerContainer";
    params: InspectDockerContainer;
} | {
    type: "GetResourceMatchingContainer";
    params: GetResourceMatchingContainer;
} | {
    type: "GetContainerLog";
    params: GetContainerLog;
} | {
    type: "SearchContainerLog";
    params: SearchContainerLog;
} | {
    type: "ListComposeProjects";
    params: ListComposeProjects;
} | {
    type: "ListDockerNetworks";
    params: ListDockerNetworks;
} | {
    type: "InspectDockerNetwork";
    params: InspectDockerNetwork;
} | {
    type: "ListDockerImages";
    params: ListDockerImages;
} | {
    type: "InspectDockerImage";
    params: InspectDockerImage;
} | {
    type: "ListDockerImageHistory";
    params: ListDockerImageHistory;
} | {
    type: "ListDockerVolumes";
    params: ListDockerVolumes;
} | {
    type: "InspectDockerVolume";
    params: InspectDockerVolume;
} | {
    type: "GetSystemInformation";
    params: GetSystemInformation;
} | {
    type: "GetSystemStats";
    params: GetSystemStats;
} | {
    type: "GetHistoricalServerStats";
    params: GetHistoricalServerStats;
} | {
    type: "ListSystemProcesses";
    params: ListSystemProcesses;
} | {
    type: "GetStacksSummary";
    params: GetStacksSummary;
} | {
    type: "GetStack";
    params: GetStack;
} | {
    type: "GetStackActionState";
    params: GetStackActionState;
} | {
    type: "GetStackLog";
    params: GetStackLog;
} | {
    type: "SearchStackLog";
    params: SearchStackLog;
} | {
    type: "InspectStackContainer";
    params: InspectStackContainer;
} | {
    type: "InspectStackSwarmService";
    params: InspectStackSwarmService;
} | {
    type: "ListStacks";
    params: ListStacks;
} | {
    type: "ListFullStacks";
    params: ListFullStacks;
} | {
    type: "ListStackServices";
    params: ListStackServices;
} | {
    type: "ListCommonStackExtraArgs";
    params: ListCommonStackExtraArgs;
} | {
    type: "ListCommonStackBuildExtraArgs";
    params: ListCommonStackBuildExtraArgs;
} | {
    type: "GetDeploymentsSummary";
    params: GetDeploymentsSummary;
} | {
    type: "GetDeployment";
    params: GetDeployment;
} | {
    type: "GetDeploymentContainer";
    params: GetDeploymentContainer;
} | {
    type: "GetDeploymentActionState";
    params: GetDeploymentActionState;
} | {
    type: "GetDeploymentStats";
    params: GetDeploymentStats;
} | {
    type: "GetDeploymentLog";
    params: GetDeploymentLog;
} | {
    type: "SearchDeploymentLog";
    params: SearchDeploymentLog;
} | {
    type: "InspectDeploymentContainer";
    params: InspectDeploymentContainer;
} | {
    type: "ListDeployments";
    params: ListDeployments;
} | {
    type: "ListFullDeployments";
    params: ListFullDeployments;
} | {
    type: "ListCommonDeploymentExtraArgs";
    params: ListCommonDeploymentExtraArgs;
} | {
    type: "GetBuildsSummary";
    params: GetBuildsSummary;
} | {
    type: "GetBuild";
    params: GetBuild;
} | {
    type: "GetBuildActionState";
    params: GetBuildActionState;
} | {
    type: "GetBuildMonthlyStats";
    params: GetBuildMonthlyStats;
} | {
    type: "ListBuildVersions";
    params: ListBuildVersions;
} | {
    type: "ListBuilds";
    params: ListBuilds;
} | {
    type: "ListFullBuilds";
    params: ListFullBuilds;
} | {
    type: "ListCommonBuildExtraArgs";
    params: ListCommonBuildExtraArgs;
} | {
    type: "GetReposSummary";
    params: GetReposSummary;
} | {
    type: "GetRepo";
    params: GetRepo;
} | {
    type: "GetRepoActionState";
    params: GetRepoActionState;
} | {
    type: "ListRepos";
    params: ListRepos;
} | {
    type: "ListFullRepos";
    params: ListFullRepos;
} | {
    type: "GetProceduresSummary";
    params: GetProceduresSummary;
} | {
    type: "GetProcedure";
    params: GetProcedure;
} | {
    type: "GetProcedureActionState";
    params: GetProcedureActionState;
} | {
    type: "ListProcedures";
    params: ListProcedures;
} | {
    type: "ListFullProcedures";
    params: ListFullProcedures;
} | {
    type: "GetActionsSummary";
    params: GetActionsSummary;
} | {
    type: "GetAction";
    params: GetAction;
} | {
    type: "GetActionActionState";
    params: GetActionActionState;
} | {
    type: "ListActions";
    params: ListActions;
} | {
    type: "ListFullActions";
    params: ListFullActions;
} | {
    type: "ListSchedules";
    params: ListSchedules;
} | {
    type: "GetResourceSyncsSummary";
    params: GetResourceSyncsSummary;
} | {
    type: "GetResourceSync";
    params: GetResourceSync;
} | {
    type: "GetResourceSyncActionState";
    params: GetResourceSyncActionState;
} | {
    type: "ListResourceSyncs";
    params: ListResourceSyncs;
} | {
    type: "ListFullResourceSyncs";
    params: ListFullResourceSyncs;
} | {
    type: "GetBuildersSummary";
    params: GetBuildersSummary;
} | {
    type: "GetBuilder";
    params: GetBuilder;
} | {
    type: "ListBuilders";
    params: ListBuilders;
} | {
    type: "ListFullBuilders";
    params: ListFullBuilders;
} | {
    type: "GetAlertersSummary";
    params: GetAlertersSummary;
} | {
    type: "GetAlerter";
    params: GetAlerter;
} | {
    type: "ListAlerters";
    params: ListAlerters;
} | {
    type: "ListFullAlerters";
    params: ListFullAlerters;
} | {
    type: "ExportAllResourcesToToml";
    params: ExportAllResourcesToToml;
} | {
    type: "ExportResourcesToToml";
    params: ExportResourcesToToml;
} | {
    type: "GetTag";
    params: GetTag;
} | {
    type: "ListTags";
    params: ListTags;
} | {
    type: "GetUsername";
    params: GetUsername;
} | {
    type: "GetPermission";
    params: GetPermission;
} | {
    type: "FindUser";
    params: FindUser;
} | {
    type: "ListUsers";
    params: ListUsers;
} | {
    type: "ListApiKeys";
    params: ListApiKeys;
} | {
    type: "ListApiKeysForServiceUser";
    params: ListApiKeysForServiceUser;
} | {
    type: "ListPermissions";
    params: ListPermissions;
} | {
    type: "ListUserTargetPermissions";
    params: ListUserTargetPermissions;
} | {
    type: "GetUserGroup";
    params: GetUserGroup;
} | {
    type: "ListUserGroups";
    params: ListUserGroups;
} | {
    type: "GetUpdate";
    params: GetUpdate;
} | {
    type: "ListUpdates";
    params: ListUpdates;
} | {
    type: "ListAlerts";
    params: ListAlerts;
} | {
    type: "GetAlert";
    params: GetAlert;
} | {
    type: "GetVariable";
    params: GetVariable;
} | {
    type: "ListVariables";
    params: ListVariables;
} | {
    type: "GetGitProviderAccount";
    params: GetGitProviderAccount;
} | {
    type: "ListGitProviderAccounts";
    params: ListGitProviderAccounts;
} | {
    type: "GetDockerRegistryAccount";
    params: GetDockerRegistryAccount;
} | {
    type: "ListDockerRegistryAccounts";
    params: ListDockerRegistryAccounts;
} | {
    type: "ListOnboardingKeys";
    params: ListOnboardingKeys;
};
export declare enum RepoWebhookAction {
    Clone = "Clone",
    Pull = "Pull",
    Build = "Build"
}
/** The specific types of permission that a User or UserGroup can have on a resource. */
export declare enum SpecificPermission {
    /**
     * On **Server**
     * - Access the terminal apis
     * On **Stack / Deployment**
     * - Access the container exec Apis
     */
    Terminal = "Terminal",
    /**
     * On **Server**
     * - Allowed to attach Stacks, Deployments, Repos, Builders to the Server
     * On **Builder**
     * - Allowed to attach Builds to the Builder
     * On **Build**
     * - Allowed to attach Deployments to the Build
     */
    Attach = "Attach",
    /**
     * On **Server**
     * - Access the `container inspect` apis
     * On **Stack / Deployment**
     * - Access `container inspect` apis for associated containers
     */
    Inspect = "Inspect",
    /**
     * On **Server**
     * - Read all container logs on the server
     * On **Stack / Deployment**
     * - Read the container logs
     */
    Logs = "Logs",
    /**
     * On **Server**
     * - Read all the processes on the host
     */
    Processes = "Processes"
}
export declare enum StackWebhookAction {
    Refresh = "Refresh",
    Deploy = "Deploy"
}
export declare enum SyncWebhookAction {
    Refresh = "Refresh",
    Sync = "Sync"
}
export type UserRequest = {
    type: "PushRecentlyViewed";
    params: PushRecentlyViewed;
} | {
    type: "SetLastSeenUpdate";
    params: SetLastSeenUpdate;
} | {
    type: "CreateApiKey";
    params: CreateApiKey;
} | {
    type: "DeleteApiKey";
    params: DeleteApiKey;
};
export type WriteRequest = {
    type: "UpdateResourceMeta";
    params: UpdateResourceMeta;
} | {
    type: "CreateSwarm";
    params: CreateSwarm;
} | {
    type: "CopySwarm";
    params: CopySwarm;
} | {
    type: "DeleteSwarm";
    params: DeleteSwarm;
} | {
    type: "UpdateSwarm";
    params: UpdateSwarm;
} | {
    type: "RenameSwarm";
    params: RenameSwarm;
} | {
    type: "CreateServer";
    params: CreateServer;
} | {
    type: "CopyServer";
    params: CopyServer;
} | {
    type: "DeleteServer";
    params: DeleteServer;
} | {
    type: "UpdateServer";
    params: UpdateServer;
} | {
    type: "RenameServer";
    params: RenameServer;
} | {
    type: "CreateNetwork";
    params: CreateNetwork;
} | {
    type: "UpdateServerPublicKey";
    params: UpdateServerPublicKey;
} | {
    type: "RotateServerKeys";
    params: RotateServerKeys;
} | {
    type: "CreateTerminal";
    params: CreateTerminal;
} | {
    type: "DeleteTerminal";
    params: DeleteTerminal;
} | {
    type: "DeleteAllTerminals";
    params: DeleteAllTerminals;
} | {
    type: "BatchDeleteAllTerminals";
    params: BatchDeleteAllTerminals;
} | {
    type: "CreateStack";
    params: CreateStack;
} | {
    type: "CopyStack";
    params: CopyStack;
} | {
    type: "DeleteStack";
    params: DeleteStack;
} | {
    type: "UpdateStack";
    params: UpdateStack;
} | {
    type: "RenameStack";
    params: RenameStack;
} | {
    type: "WriteStackFileContents";
    params: WriteStackFileContents;
} | {
    type: "RefreshStackCache";
    params: RefreshStackCache;
} | {
    type: "CreateDeployment";
    params: CreateDeployment;
} | {
    type: "CopyDeployment";
    params: CopyDeployment;
} | {
    type: "CreateDeploymentFromContainer";
    params: CreateDeploymentFromContainer;
} | {
    type: "DeleteDeployment";
    params: DeleteDeployment;
} | {
    type: "UpdateDeployment";
    params: UpdateDeployment;
} | {
    type: "RenameDeployment";
    params: RenameDeployment;
} | {
    type: "CreateBuild";
    params: CreateBuild;
} | {
    type: "CopyBuild";
    params: CopyBuild;
} | {
    type: "DeleteBuild";
    params: DeleteBuild;
} | {
    type: "UpdateBuild";
    params: UpdateBuild;
} | {
    type: "RenameBuild";
    params: RenameBuild;
} | {
    type: "WriteBuildFileContents";
    params: WriteBuildFileContents;
} | {
    type: "RefreshBuildCache";
    params: RefreshBuildCache;
} | {
    type: "CreateRepo";
    params: CreateRepo;
} | {
    type: "CopyRepo";
    params: CopyRepo;
} | {
    type: "DeleteRepo";
    params: DeleteRepo;
} | {
    type: "UpdateRepo";
    params: UpdateRepo;
} | {
    type: "RenameRepo";
    params: RenameRepo;
} | {
    type: "RefreshRepoCache";
    params: RefreshRepoCache;
} | {
    type: "CreateProcedure";
    params: CreateProcedure;
} | {
    type: "CopyProcedure";
    params: CopyProcedure;
} | {
    type: "DeleteProcedure";
    params: DeleteProcedure;
} | {
    type: "UpdateProcedure";
    params: UpdateProcedure;
} | {
    type: "RenameProcedure";
    params: RenameProcedure;
} | {
    type: "CreateAction";
    params: CreateAction;
} | {
    type: "CopyAction";
    params: CopyAction;
} | {
    type: "DeleteAction";
    params: DeleteAction;
} | {
    type: "UpdateAction";
    params: UpdateAction;
} | {
    type: "RenameAction";
    params: RenameAction;
} | {
    type: "CreateResourceSync";
    params: CreateResourceSync;
} | {
    type: "CopyResourceSync";
    params: CopyResourceSync;
} | {
    type: "DeleteResourceSync";
    params: DeleteResourceSync;
} | {
    type: "UpdateResourceSync";
    params: UpdateResourceSync;
} | {
    type: "RenameResourceSync";
    params: RenameResourceSync;
} | {
    type: "WriteSyncFileContents";
    params: WriteSyncFileContents;
} | {
    type: "CommitSync";
    params: CommitSync;
} | {
    type: "RefreshResourceSyncPending";
    params: RefreshResourceSyncPending;
} | {
    type: "CreateBuilder";
    params: CreateBuilder;
} | {
    type: "CopyBuilder";
    params: CopyBuilder;
} | {
    type: "DeleteBuilder";
    params: DeleteBuilder;
} | {
    type: "UpdateBuilder";
    params: UpdateBuilder;
} | {
    type: "RenameBuilder";
    params: RenameBuilder;
} | {
    type: "CreateAlerter";
    params: CreateAlerter;
} | {
    type: "CopyAlerter";
    params: CopyAlerter;
} | {
    type: "DeleteAlerter";
    params: DeleteAlerter;
} | {
    type: "UpdateAlerter";
    params: UpdateAlerter;
} | {
    type: "RenameAlerter";
    params: RenameAlerter;
} | {
    type: "CreateOnboardingKey";
    params: CreateOnboardingKey;
} | {
    type: "UpdateOnboardingKey";
    params: UpdateOnboardingKey;
} | {
    type: "DeleteOnboardingKey";
    params: DeleteOnboardingKey;
} | {
    type: "CreateLocalUser";
    params: CreateLocalUser;
} | {
    type: "UpdateUserUsername";
    params: UpdateUserUsername;
} | {
    type: "UpdateUserPassword";
    params: UpdateUserPassword;
} | {
    type: "DeleteUser";
    params: DeleteUser;
} | {
    type: "CreateServiceUser";
    params: CreateServiceUser;
} | {
    type: "UpdateServiceUserDescription";
    params: UpdateServiceUserDescription;
} | {
    type: "CreateApiKeyForServiceUser";
    params: CreateApiKeyForServiceUser;
} | {
    type: "DeleteApiKeyForServiceUser";
    params: DeleteApiKeyForServiceUser;
} | {
    type: "CreateUserGroup";
    params: CreateUserGroup;
} | {
    type: "RenameUserGroup";
    params: RenameUserGroup;
} | {
    type: "DeleteUserGroup";
    params: DeleteUserGroup;
} | {
    type: "AddUserToUserGroup";
    params: AddUserToUserGroup;
} | {
    type: "RemoveUserFromUserGroup";
    params: RemoveUserFromUserGroup;
} | {
    type: "SetUsersInUserGroup";
    params: SetUsersInUserGroup;
} | {
    type: "SetEveryoneUserGroup";
    params: SetEveryoneUserGroup;
} | {
    type: "UpdateUserAdmin";
    params: UpdateUserAdmin;
} | {
    type: "UpdateUserBasePermissions";
    params: UpdateUserBasePermissions;
} | {
    type: "UpdatePermissionOnResourceType";
    params: UpdatePermissionOnResourceType;
} | {
    type: "UpdatePermissionOnTarget";
    params: UpdatePermissionOnTarget;
} | {
    type: "CreateTag";
    params: CreateTag;
} | {
    type: "DeleteTag";
    params: DeleteTag;
} | {
    type: "RenameTag";
    params: RenameTag;
} | {
    type: "UpdateTagColor";
    params: UpdateTagColor;
} | {
    type: "CreateVariable";
    params: CreateVariable;
} | {
    type: "UpdateVariableValue";
    params: UpdateVariableValue;
} | {
    type: "UpdateVariableDescription";
    params: UpdateVariableDescription;
} | {
    type: "UpdateVariableIsSecret";
    params: UpdateVariableIsSecret;
} | {
    type: "DeleteVariable";
    params: DeleteVariable;
} | {
    type: "CreateGitProviderAccount";
    params: CreateGitProviderAccount;
} | {
    type: "UpdateGitProviderAccount";
    params: UpdateGitProviderAccount;
} | {
    type: "DeleteGitProviderAccount";
    params: DeleteGitProviderAccount;
} | {
    type: "CreateDockerRegistryAccount";
    params: CreateDockerRegistryAccount;
} | {
    type: "UpdateDockerRegistryAccount";
    params: UpdateDockerRegistryAccount;
} | {
    type: "DeleteDockerRegistryAccount";
    params: DeleteDockerRegistryAccount;
} | {
    type: "CloseAlert";
    params: CloseAlert;
};
export type WsLoginMessage = {
    type: "Jwt";
    params: {
        jwt: string;
    };
} | {
    type: "ApiKeys";
    params: {
        key: string;
        secret: string;
    };
};
