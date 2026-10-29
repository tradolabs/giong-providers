import {ResourceConfig} from "@giong/sdk";

export interface WorkerDomainConfig {
    hostname: string;
    zoneId: string;
    service: string;
}

export class WorkerDomain extends ResourceConfig {
    readonly hostname: string;
    readonly zoneId: string;
    readonly service: string;

    constructor(config: WorkerDomainConfig) {
        super();
        this.hostname = config.hostname;
        this.zoneId = config.zoneId;
        this.service = config.service;
    }

    get type(): string {
        return "cloudflare.worker-domain";
    }
}
