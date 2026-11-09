export namespace models {
	
	export class Collection {
	    id: number;
	    parent_id?: number;
	    name: string;
	    // Go type: time
	    created_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Collection(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.parent_id = source["parent_id"];
	        this.name = source["name"];
	        this.created_at = this.convertValues(source["created_at"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class Environment {
	    id: number;
	    name: string;
	    is_active: boolean;
	    // Go type: time
	    created_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Environment(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.is_active = source["is_active"];
	        this.created_at = this.convertValues(source["created_at"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class KV {
	    key: string;
	    value: string;
	    enable: boolean;
	
	    static createFrom(source: any = {}) {
	        return new KV(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.key = source["key"];
	        this.value = source["value"];
	        this.enable = source["enable"];
	    }
	}
	export class Request {
	    id: number;
	    collection_id?: number;
	    name: string;
	    method: string;
	    url: string;
	    headers: KV[];
	    params: KV[];
	    body_type: string;
	    body: string;
	    tests_script: string;
	    // Go type: time
	    created_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Request(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.collection_id = source["collection_id"];
	        this.name = source["name"];
	        this.method = source["method"];
	        this.url = source["url"];
	        this.headers = this.convertValues(source["headers"], KV);
	        this.params = this.convertValues(source["params"], KV);
	        this.body_type = source["body_type"];
	        this.body = source["body"];
	        this.tests_script = source["tests_script"];
	        this.created_at = this.convertValues(source["created_at"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class Response {
	    status_code: number;
	    status: string;
	    time_ms: number;
	    req_size: number;
	    resp_size: number;
	    headers: Record<string, string>;
	    body: string;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new Response(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.status_code = source["status_code"];
	        this.status = source["status"];
	        this.time_ms = source["time_ms"];
	        this.req_size = source["req_size"];
	        this.resp_size = source["resp_size"];
	        this.headers = source["headers"];
	        this.body = source["body"];
	        this.error = source["error"];
	    }
	}
	export class TestResult {
	    request_id: number;
	    name: string;
	    passed: boolean;
	    status_code: number;
	    time_ms: number;
	    asserts: number;
	    failed_msg?: string;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new TestResult(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.request_id = source["request_id"];
	        this.name = source["name"];
	        this.passed = source["passed"];
	        this.status_code = source["status_code"];
	        this.time_ms = source["time_ms"];
	        this.asserts = source["asserts"];
	        this.failed_msg = source["failed_msg"];
	        this.error = source["error"];
	    }
	}
	export class Variable {
	    id: number;
	    environment_id: number;
	    key: string;
	    value: string;
	    type: string;
	
	    static createFrom(source: any = {}) {
	        return new Variable(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.environment_id = source["environment_id"];
	        this.key = source["key"];
	        this.value = source["value"];
	        this.type = source["type"];
	    }
	}

}

