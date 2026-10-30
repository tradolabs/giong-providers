import {buildSync} from "esbuild";
import {ResourceConfig} from "@giong/sdk";

export interface WorkerContainerConfig {
    image: string;
    className: string;
    binding: string;
    maxInstances?: number;
    instanceType?: string;
}

export interface WorkerConfig {
    name: string;
    main: string;
    container?: WorkerContainerConfig;
    compatibilityDate: string;
    observability?: boolean;
    vars?: Record<string, string>;
}

export class Worker extends ResourceConfig {
    readonly name: string;
    readonly source: string;
    readonly container?: WorkerContainerConfig & {maxInstances: number; instanceType: string};
    readonly compatibilityDate: string;
    readonly observability: boolean;
    readonly vars: Record<string, string>;

    constructor(config: WorkerConfig) {
        super();
        this.name = config.name;
        this.container = config.container === undefined ? undefined : {
            ...config.container,
            maxInstances: config.container.maxInstances ?? 1,
            instanceType: config.container.instanceType ?? "lite",
        };
        this.compatibilityDate = config.compatibilityDate;
        this.observability = config.observability ?? true;
        this.vars = config.vars ?? {};
        this.source = buildSync({
            entryPoints: [config.main],
            bundle: true,
            write: false,
            format: "esm",
            platform: "neutral",
            target: "es2022",
            conditions: ["workerd", "worker", "browser"],
            external: ["cloudflare:*", "node:*"],
        }).outputFiles[0].text;
    }

    get type(): string {
        return "cloudflare.worker";
    }
}
