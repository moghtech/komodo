import { KomodoClient as Client } from "./client/lib.js";
import "./deno.d.ts";
import "./client/types_namespace.d.ts";

declare global {
  // =================
  // 🔴 Docker Compose
  // =================

  /**
   * Docker Compose configuration interface
   */
  export interface DockerCompose {
    /** Version of the Compose file format */
    version?: string;
    /** Defines services within the Docker Compose file */
    services: Record<string, DockerComposeService>;
    /** Defines volumes in the Docker Compose file */
    volumes?: Record<string, DockerComposeVolume>;
    /** Defines networks in the Docker Compose file */
    networks?: Record<string, DockerComposeNetwork>;
  }

  /**
   * Describes a service within Docker Compose
   */
  export interface DockerComposeService {
    /** Docker image to use */
    image?: string;
    /** Build configuration for the service */
    build?: DockerComposeServiceBuild;
    /** Ports to map, supporting single strings or mappings */
    ports?: (string | DockerComposeServicePortMapping)[];
    /** Environment variables to set within the container */
    environment?: Record<string, string>;
    /** Volumes to mount */
    volumes?: (string | DockerComposeServiceVolumeMount)[];
    /** Networks to attach the service to */
    networks?: string[];
    /** Dependencies of the service */
    depends_on?: string[];
    /** Command to override the default CMD */
    command?: string | string[];
    /** Entrypoint to override the default ENTRYPOINT */
    entrypoint?: string | string[];
    /** Container name */
    container_name?: string;
    /** Healthcheck configuration for the service */
    healthcheck?: DockerComposeServiceHealthcheck;
    /** Logging options for the service */
    logging?: DockerComposeServiceLogging;
    /** Deployment settings for the service */
    deploy?: DockerComposeServiceDeploy;
    /** Restart policy */
    restart?: string;
    /** Security options */
    security_opt?: string[];
    /** Ulimits configuration */
    ulimits?: Record<string, DockerComposeServiceUlimit>;
    /** Secrets to be used by the service */
    secrets?: string[];
    /** Configuration items */
    configs?: string[];
    /** Labels to apply to the service */
    labels?: Record<string, string>;
    /** Number of CPU units assigned */
    cpus?: string | number;
    /** Memory limit */
    mem_limit?: string;
    /** CPU shares for container allocation */
    cpu_shares?: number;
    /** Extra hosts for the service */
    extra_hosts?: string[];
    [key: string]: unknown;
  }

  /**
   * Configuration for Docker build
   */
  export interface DockerComposeServiceBuild {
    /** Build context path */
    context: string;
    /** Dockerfile path within the context */
    dockerfile?: string;
    /** Build arguments to pass */
    args?: Record<string, string>;
    /** Sources for cache imports */
    cache_from?: string[];
    /** Labels for the build */
    labels?: Record<string, string>;
    /** Network mode for build process */
    network?: string;
    /** Target build stage */
    target?: string;
    /** Shared memory size */
    shm_size?: string;
    /** Secrets for the build process */
    secrets?: string[];
    /** Extra hosts for build process */
    extra_hosts?: string[];
  }

  /**
   * Port mapping configuration
   */
  export interface DockerComposeServicePortMapping {
    /** Target port inside the container */
    target: number;
    /** Published port on the host */
    published?: number;
    /** Protocol used for the port (tcp/udp) */
    protocol?: "tcp" | "udp";
    /** Mode for port publishing */
    mode?: "host" | "ingress";
  }

  /**
   * Volume mount configuration
   */
  export interface DockerComposeServiceVolumeMount {
    /** Type of volume mount */
    type: "volume" | "bind" | "tmpfs";
    /** Source path or name */
    source: string;
    /** Target path within the container */
    target: string;
    /** Whether the volume is read-only */
    read_only?: boolean;
  }

  /**
   * Healthcheck configuration for a service
   */
  export interface DockerComposeServiceHealthcheck {
    /** Command to check health */
    test: string | string[];
    /** Interval between checks */
    interval?: string;
    /** Timeout for each check */
    timeout?: string;
    /** Maximum number of retries */
    retries?: number;
    /** Initial delay before checks start */
    start_period?: string;
  }

  /**
   * Logging configuration for a service
   */
  export interface DockerComposeServiceLogging {
    /** Logging driver */
    driver: string;
    /** Options for the logging driver */
    options?: Record<string, string>;
  }

  /**
   * Deployment configuration for a service
   */
  export interface DockerComposeServiceDeploy {
    /** Number of replicas */
    replicas?: number;
    /** Update configuration */
    update_config?: DockerComposeServiceDeploy;
    /** Restart policy */
    restart_policy?: DockerComposeServiceDeployRestartPolicy;
  }

  /**
   * Update configuration during deployment
   */
  export interface DockerComposeServiceDeployUpdateConfig {
    /** Number of containers updated in parallel */
    parallelism?: number;
    /** Delay between updates */
    delay?: string;
    /** Action on failure */
    failure_action?: string;
    /** Order of updates */
    order?: string;
  }

  /**
   * Restart policy configuration
   */
  export interface DockerComposeServiceDeployRestartPolicy {
    /** Condition for restart */
    condition: "none" | "on-failure" | "any";
    /** Delay before restarting */
    delay?: string;
    /** Maximum number of restart attempts */
    max_attempts?: number;
    /** Time window for restart attempts */
    window?: string;
  }

  /**
   * Ulimit configuration
   */
  export interface DockerComposeServiceUlimit {
    /** Soft limit */
    soft: number;
    /** Hard limit */
    hard: number;
  }

  /**
   * Volume configuration in Docker Compose
   */
  export interface DockerComposeVolume {
    /** Volume driver to use */
    driver?: string;
    /** Driver options */
    driver_opts?: Record<string, string>;
    /** External volume identifier */
    external?: boolean | string;
  }

  /**
   * Network configuration in Docker Compose
   */
  export interface DockerComposeNetwork {
    /** Network driver */
    driver?: string;
    /** Indicates if network is external */
    external?: boolean;
  }

  // =====================
  // 🔴 YAML De/serializer
  // =====================

  // https://jsr.io/@std/yaml

  export type YamlSchemaType =
    | "failsafe"
    | "json"
    | "core"
    | "default"
    | "extended";

  export type YamlStyleVariant =
    | "lowercase"
    | "uppercase"
    | "camelcase"
    | "decimal"
    | "binary"
    | "octal"
    | "hexadecimal";

  /** Options for `YAML.stringify` */
  export type YamlStringifyOptions = {
    /**
     * Indentation width to use (in spaces).
     *
     * @default {2}
     */
    indent?: number;
    /**
     * When true, adds an indentation level to array elements.
     *
     * @default {true}
     */
    arrayIndent?: boolean;
    /**
     * Do not throw on invalid types (like function in the safe schema) and skip
     * pairs and single values with such types.
     *
     * @default {false}
     */
    skipInvalid?: boolean;
    /**
     * Specifies level of nesting, when to switch from block to flow style for
     * collections. `-1` means block style everywhere.
     *
     * @default {-1}
     */
    flowLevel?: number;
    /** Each tag may have own set of styles.	- "tag" => "style" map. */
    styles?: Record<string, YamlStyleVariant>;
    /**
     * Name of the schema to use.
     *
     * @default {"default"}
     */
    schema?: YamlSchemaType;
    /**
     * If true, sort keys when dumping YAML in ascending, ASCII character order.
     * If a function, use the function to sort the keys.
     * If a function is specified, the function must return a negative value
     * if first argument is less than second argument, zero if they're equal
     * and a positive value otherwise.
     *
     * @default {false}
     */
    sortKeys?: boolean | ((a: string, b: string) => number);
    /**
     * Set max line width.
     *
     * @default {80}
     */
    lineWidth?: number;
    /**
     * If false, don't convert duplicate objects into references.
     *
     * @default {true}
     */
    useAnchors?: boolean;
    /**
     * If false don't try to be compatible with older yaml versions.
     * Currently: don't quote "yes", "no" and so on,
     * as required for YAML 1.1.
     *
     * @default {true}
     */
    compatMode?: boolean;
    /**
     * If true flow sequences will be condensed, omitting the
     * space between `key: value` or `a, b`. Eg. `'[a,b]'` or `{a:{b:c}}`.
     * Can be useful when using yaml for pretty URL query params
     * as spaces are %-encoded.
     *
     * @default {false}
     */
    condenseFlow?: boolean;
  };

  /** Options for `YAML.parse` */
  export interface YamlParseOptions {
    /**
     * Name of the schema to use.
     *
     * @default {"default"}
     */
    schema?: YamlSchemaType;
    /**
     * If `true`, duplicate keys will overwrite previous values. Otherwise,
     * duplicate keys will throw a {@linkcode SyntaxError}.
     *
     * @default {false}
     */
    allowDuplicateKeys?: boolean;
    /**
     * If defined, a function to call on warning messages taking an
     * {@linkcode Error} as its only argument.
     */
    onWarning?(error: Error): void;
  }

  // ===============
  // 🔴 Cargo TOML 🦀
  // ===============

  /**
   * Represents the structure of a Cargo.toml manifest file.
   */
  export interface CargoToml {
    /**
     * Information about the main package in the Cargo project.
     */
    package?: CargoTomlPackage;

    /**
     * Dependencies required by the project, organized into normal, development, and build dependencies.
     */
    dependencies?: CargoTomlDependencies;

    /**
     * Development dependencies required by the project.
     */
    devDependencies?: CargoTomlDependencies;

    /**
     * Build dependencies required by the project.
     */
    buildDependencies?: CargoTomlDependencies;

    /**
     * Features available in the package, each as an array of dependency names or other features.
     */
    features?: Record<string, string[]>;

    /**
     * Build profiles available in the package, allowing for profile-specific configurations.
     */
    profile?: CargoTomlProfiles;

    /**
     * Path to the custom build script for the package, if applicable.
     */
    build?: string;

    /**
     * Workspace configuration for multi-package Cargo projects.
     */
    workspace?: CargoTomlWorkspace;

    /**
     * Additional metadata ignored by Cargo but potentially used by other tools.
     */
    [key: string]: any;
  }

  /**
   * Metadata for the main package in the Cargo project.
   */
  export interface CargoTomlPackage {
    /**
     * The name of the package, used by Cargo and for crate publishing.
     */
    name: string;

    /**
     * The version of the package, following Semantic Versioning.
     */
    version: string;

    /**
     * List of author names or emails.
     */
    authors?: string[];

    /**
     * The Rust edition for this package.
     */
    edition?: "2015" | "2018" | "2021";

    /**
     * Short description of the package.
     */
    description?: string;

    /**
     * The license for the package, specified as a SPDX identifier.
     */
    license?: string;

    /**
     * Path to a custom license file for the package.
     */
    licenseFile?: string;

    /**
     * URL to the package documentation.
     */
    documentation?: string;

    /**
     * URL to the package homepage.
     */
    homepage?: string;

    /**
     * URL to the package repository.
     */
    repository?: string;

    /**
     * Path to the README file for the package.
     */
    readme?: string;

    /**
     * List of keywords for the package, used for search optimization.
     */
    keywords?: string[];

    /**
     * List of categories that the package belongs to.
     */
    categories?: string[];

    /**
     * Workspace that this package belongs to, if any.
     */
    workspace?: string;

    /**
     * Path to a build script for the package.
     */
    build?: string;

    /**
     * Name of a native library to link with, if applicable.
     */
    links?: string;

    /**
     * List of paths to exclude from the package.
     */
    exclude?: string[];

    /**
     * List of paths to include in the package.
     */
    include?: string[];

    /**
     * Indicates whether the package should be published to crates.io.
     */
    publish?: boolean;

    /**
     * Arbitrary metadata that is ignored by Cargo but can be used by other tools.
     */
    metadata?: Record<string, any>;

    /**
     * Auto-enable binaries for the package.
     */
    autobins?: boolean;

    /**
     * Auto-enable examples for the package.
     */
    autoexamples?: boolean;

    /**
     * Auto-enable tests for the package.
     */
    autotests?: boolean;

    /**
     * Auto-enable benchmarks for the package.
     */
    autobenches?: boolean;

    /**
     * Specifies the version of dependency resolution to use.
     */
    resolver?: "1" | "2";
  }

  /**
   * A map of dependencies in the Cargo manifest, with each dependency represented by its name.
   */
  export type CargoTomlDependencies = Record<string, CargoTomlDependency>;

  /**
   * Information about a specific dependency in the Cargo manifest.
   */
  export type CargoTomlDependency =
    | string
    | {
        /**
         * Version requirement for the dependency.
         */
        version?: string;

        /**
         * Path to a local dependency.
         */
        path?: string;

        /**
         * Name of the registry to use for this dependency.
         */
        registry?: string;

        /**
         * URL to a Git repository for this dependency.
         */
        git?: string;

        /**
         * Branch to use for a Git dependency.
         */
        branch?: string;

        /**
         * Tag to use for a Git dependency.
         */
        tag?: string;

        /**
         * Specific revision to use for a Git dependency.
         */
        rev?: string;

        /**
         * Marks this dependency as optional.
         */
        optional?: boolean;

        /**
         * Enables default features for this dependency.
         */
        defaultFeatures?: boolean;

        /**
         * List of features to enable for this dependency.
         */
        features?: string[];

        /**
         * Renames the dependency package name.
         */
        package?: string;
      };

  /**
   * Defines available profiles for building the package.
   */
  export interface CargoTomlProfiles {
    /**
     * Development profile configuration.
     */
    dev?: CargoTomlProfile;

    /**
     * Release profile configuration.
     */
    release?: CargoTomlProfile;

    /**
     * Test profile configuration.
     */
    test?: CargoTomlProfile;

    /**
     * Benchmark profile configuration.
     */
    bench?: CargoTomlProfile;

    /**
     * Documentation profile configuration.
     */
    doc?: CargoTomlProfile;

    /**
     * Additional custom profiles.
     */
    [profileName: string]: CargoTomlProfile | undefined;
  }

  /**
   * Configuration for an individual build profile.
   */
  export interface CargoTomlProfile {
    /**
     * Profile that this profile inherits from.
     */
    inherits?: string;

    /**
     * Optimization level for the profile.
     */
    optLevel?: "0" | "1" | "2" | "3" | "s" | "z";

    /**
     * Enables debug information, either as a boolean or a level.
     */
    debug?: boolean | number;

    /**
     * Controls how debug information is split.
     */
    splitDebugInfo?: "unpacked" | "packed" | "off";

    /**
     * Enables or disables debug assertions.
     */
    debugAssertions?: boolean;

    /**
     * Enables or disables overflow checks.
     */
    overflowChecks?: boolean;

    /**
     * Enables or disables unit testing for the profile.
     */
    test?: boolean;

    /**
     * Link-time optimization settings for the profile.
     */
    lto?: boolean | "thin" | "fat";

    /**
     * Panic strategy for the profile.
     */
    panic?: "unwind" | "abort";

    /**
     * Enables or disables incremental compilation.
     */
    incremental?: boolean;

    /**
     * Number of code generation units for parallelism.
     */
    codegenUnits?: number;

    /**
     * Enables or disables the use of runtime paths.
     */
    rpath?: boolean;

    /**
     * Specifies stripping options for the binary.
     */
    strip?: boolean | "debuginfo" | "symbols";

    /**
     * Additional custom profile fields.
     */
    [key: string]: any;
  }

  /**
   * Defines workspace-specific settings for a Cargo project.
   */
  export interface CargoTomlWorkspace {
    /**
     * Members of the workspace.
     */
    members?: string[];

    /**
     * Paths to exclude from the workspace.
     */
    exclude?: string[];

    /**
     * Members to include by default when building the workspace.
     */
    defaultMembers?: string[];

    /**
     * Common Information about the packages in the Cargo workspace.
     */
    package?: CargoTomlPackage;

    /**
     * Additional custom workspace fields.
     */
    [key: string]: any;
  }

  // =====================
  // 🔴 TOML De/serializer
  // =====================

  // https://jsr.io/@std/toml

  export interface TomlStringifyOptions {
    /**
     * Define if the keys should be aligned or not.
     *
     * @default {false}
     */
    keyAlignment?: boolean;
  }

  /** Pre initialized Komodo client */
  var komodo: ReturnType<typeof Client>;
  /** KomodoClient initializer */
  var KomodoClient: typeof Client;
  /** The incoming arguments */
  var ARGS: {
    WEBHOOK_BRANCH?: string;
    WEBHOOK_BODY?: any;
  } & Record<string, any>;
  /** YAML parsing utilities */
  var YAML: {
    /**
     * Converts a JavaScript object or value to a YAML document string.
     *
     * @example Usage
     * ```ts
     * const data = { id: 1, name: "Alice" };
     *
     * const yaml = YAML.stringify(data);
     *
     * assertEquals(yaml, "id: 1\nname: Alice\n");
     * ```
     *
     * @throws {TypeError} If `data` contains invalid types.
     * @param data The data to serialize.
     * @param options The options for serialization.
     * @returns A YAML string.
     */
    stringify: (data: unknown, options?: YamlStringifyOptions) => string;
    /**
     * Parse and return a YAML string as a parsed YAML document object.
     *
     * Note: This does not support functions. Untrusted data is safe to parse.
     *
     * @example Usage
     * ```ts
     * const data = YAML.parse(`
     * id: 1
     * name: Alice
     * `);
     *
     * assertEquals(data, { id: 1, name: "Alice" });
     * ```
     *
     * @throws {SyntaxError} Throws error on invalid YAML.
     * @param content YAML string to parse.
     * @param options Parsing options.
     * @returns Parsed document.
     */
    parse: (content: string, options?: YamlParseOptions) => unknown;
    /**
     * Same as `YAML.parse`, but understands multi-document YAML sources, and
     * returns multiple parsed YAML document objects.
     *
     * @example Usage
     * ```ts
     * const data = YAML.parseAll(`
     * ---
     * id: 1
     * name: Alice
     * ---
     * id: 2
     * name: Bob
     * ---
     * id: 3
     * name: Eve
     * `);
     *
     * assertEquals(data, [ { id: 1, name: "Alice" }, { id: 2, name: "Bob" }, { id: 3, name: "Eve" }]);
     * ```
     *
     * @param content YAML string to parse.
     * @param options Parsing options.
     * @returns Array of parsed documents.
     */
    parseAll: (content: string, options?: YamlParseOptions) => unknown;
    /**
     * Parse and return a YAML string as a Docker Compose file.
     *
     * @example Usage
     * ```ts
     * const stack = await komodo.read("GetStack", { stack: "test-stack" });
     * const contents = stack?.config?.file_contents;
     *
     * const parsed: DockerCompose = YAML.parseDockerCompose(contents)
     * ```
     *
     * @throws {SyntaxError} Throws error on invalid YAML.
     * @param content Docker compose file string.
     * @param options Parsing options.
     * @returns Parsed document.
     */
    parseDockerCompose: (
      content: string,
      options?: YamlParseOptions
    ) => DockerCompose;
  };
  /** TOML parsing utilities */
  var TOML: {
    /**
     * Converts an object to a [TOML string](https://toml.io).
     *
     * @example Usage
     * ```ts
     * const obj = {
     *   title: "TOML Example",
     *   owner: {
     *     name: "Bob",
     *     bio: "Bob is a cool guy",
     *  }
     * };
     *
     * const tomlString = TOML.stringify(obj);
     *
     * assertEquals(tomlString, `title = "TOML Example"\n\n[owner]\nname = "Bob"\nbio = "Bob is a cool guy"\n`);
     * ```
     * @param obj Source object
     * @param options Options for stringifying.
     * @returns TOML string
     */
    stringify: (
      obj: Record<string, unknown>,
      options?: TomlStringifyOptions
    ) => string;
    /**
     * Parses a [TOML string](https://toml.io) into an object.
     *
     * @example Usage
     * ```ts
     * const tomlString = `title = "TOML Example"
     * [owner]
     * name = "Alice"
     * bio = "Alice is a programmer."`;
     *
     * const obj = TOML.parse(tomlString);
     *
     * assertEquals(obj, { title: "TOML Example", owner: { name: "Alice", bio: "Alice is a programmer." } });
     * ```
     * @param tomlString TOML string to be parsed.
     * @returns The parsed JS object.
     */
    parse: (tomlString: string) => Record<string, unknown>;
    /**
     * Parses Komodo resource.toml contents to an object
     * for easier handling.
     *
     * @example Usage
     * ```ts
     * const sync = await komodo.read("GetResourceSync", { sync: "test-sync" })
     * const contents = sync?.config?.file_contents;
     *
     * const resources: Types.ResourcesToml = TOML.parseResourceToml(contents);
     * ```
     *
     * @param resourceToml The resource file contents.
     * @returns Komodo resource.toml contents as JSON
     */
    parseResourceToml: (resourceToml: string) => Types.ResourcesToml;
    /**
     * Parses Cargo.toml contents to an object
     * for easier handling.
     *
     * @example Usage
     * ```ts
     *  const contents = Deno.readTextFile("/path/to/Cargo.toml");
     * const cargoToml: CargoToml = TOML.parseCargoToml(contents);
     * ```
     *
     * @param cargoToml The Cargo.toml contents.
     * @returns Cargo.toml contents as JSON
     */
    parseCargoToml: (cargoToml: string) => CargoToml;
  };
}
