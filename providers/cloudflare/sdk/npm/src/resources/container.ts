import {Resource} from "@giong/sdk";
import {CloudflareProvider} from "../provider.ts";

export interface ContainerConfig {
    workerName: string;
    className: string;
    image: string;
    maxInstances?: number;
    instanceType?: string;
    observability?: boolean;
}

export class Container extends Resource<CloudflareProvider> {
    private config: ContainerConfig;

    constructor(name: string, config: ContainerConfig, provider: CloudflareProvider) {
        super(name, provider);
        this.config = config;
    }

    get package(): string {
        return "cloudflare/container";
    }

    get inputs(): Record<string, unknown> {
        return {
            name: this.name,
            worker_name: this.config.workerName,
            class_name: this.config.className,
            image: this.config.image,
            max_instances: this.config.maxInstances ?? 1,
            instance_type: this.config.instanceType ?? 'lite',
            observability: this.config.observability ?? true,
        }
    }
}
