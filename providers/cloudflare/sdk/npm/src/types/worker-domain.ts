import {ResourceConfig} from "@giong/sdk";

export interface WorkerDomainConfig {
    hostname: string;
    zoneName: string;
    service: string;
}

export class WorkerDomain extends ResourceConfig {
    readonly hostname: string;
    readonly zoneName: string;
    readonly service: string;

    constructor(config: WorkerDomainConfig) {
        super();
        this.hostname = config.hostname;
        this.zoneName = config.zoneName;
        this.service = config.service;
    }

    get type(): string {
        return "cloudflare.worker-domain";
    }
}
