import {buildSync} from "esbuild";
import {ResourceConfig} from "@giong/sdk";

export interface ContainersConfig {
    name: string;
    main: string;
    image: string;
    className: string;
    binding: string;
    compatibilityDate: string;
    observability?: {enabled: boolean};
    maxInstances?: number;
    instanceType?: string;
    vars?: Record<string, string>;
}

export class Containers extends ResourceConfig {
    readonly name: string;
    readonly source: string;
    readonly image: string;
    readonly className: string;
    readonly binding: string;
    readonly compatibilityDate: string;
    readonly observability: {enabled: boolean};
    readonly maxInstances: number;
    readonly instanceType: string;
    readonly vars: Record<string, string>;

    constructor(config: ContainersConfig) {
        super();
        this.name = config.name;
        this.image = config.image;
        this.className = config.className;
        this.binding = config.binding;
        this.compatibilityDate = config.compatibilityDate;
        this.observability = config.observability ?? {enabled: true};
        this.maxInstances = config.maxInstances ?? 1;
        this.instanceType = config.instanceType ?? "lite";
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
        return "cloudflare.containers";
    }
}
