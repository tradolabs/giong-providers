import {Resource} from "@giong/sdk";
import {CloudflareProvider} from "../provider.ts";

export interface WorkerDomainConfig {
    hostname: string;
    zoneName: string;
    service: string;
}

export class WorkerDomain extends Resource<CloudflareProvider> {
    private config: WorkerDomainConfig;

    constructor(name: string, config: WorkerDomainConfig, provider: CloudflareProvider) {
        super(name, provider);
        this.config = config;
    }

    get package(): string {
        return "cloudflare/worker-domain";
    }

    get inputs(): Record<string, unknown> {
        return {
            hostname: this.config.hostname,
            zone_name: this.config.zoneName,
            service: this.config.service,
        };
    }
}
