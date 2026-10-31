import {buildSync} from "esbuild";
import {Resource} from "@giong/sdk";
import {CloudflareProvider} from "../provider.ts";

export interface WorkerContainerBinding {
    className: string;
    binding: string;
}

export interface WorkerConfig {
    main: string;
    containers?: WorkerContainerBinding[];
    compatibilityDate: string;
    observability?: boolean;
    vars?: Record<string, string>;
}

export class Worker extends Resource<CloudflareProvider> {
    private config: WorkerConfig;
    private readonly source: string;

    constructor(name: string, config: WorkerConfig, provider: CloudflareProvider) {
        super(name, provider);
        this.config = config;
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

    get package(): string {
        return "cloudflare/worker";
    }

    get inputs(): Record<string, unknown> {
        return {
            name: this.name,
            source: this.source,
            containers: (this.config.containers ?? []).map(container => ({
                class_name: container.className,
                binding: container.binding,
            })),
            compatibility_date: this.config.compatibilityDate,
            observability: this.config.observability ?? true,
            vars: this.config.vars ?? {},
        };
    }
}
