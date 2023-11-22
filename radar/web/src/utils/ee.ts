import {
    EventEmitter2 as EmitterImplementation,
} from "eventemitter2";

type StringDomain<V> = V extends `${infer N}.${infer S}` ? `${N}.*` | `${N}.${StringDomain<S>}` : V;
type EventDomains<Events> = {
    [E in keyof Events]: StringDomain<E>
}[keyof Events];

type EventPayload<Payload> = Payload extends Array<any> ? Payload : [Payload];
type ListenerPayload<Events, Event> = {
    [E in keyof Events]: Event extends `${infer N}.*` ?
    E extends `${N}.${infer S}` ? EventPayload<Events[E]> : never : /* Event is a namespace event */
    E extends Event ? EventPayload<Events[E]> : never /* Event is a key of Events */
}[keyof Events];

type EventListener = {
    /** Unregister the listener */
    (): void;
    unregister(): void;

    target: string;
    listener: Function;
};

export interface IEventEmitter<Events> {
    emit<Event extends keyof Events>(event: Event, ...values: EventPayload<Events[Event]>): void;
    emitAsync<Event extends keyof Events>(event: Event, ...values: EventPayload<Events[Event]>): Promise<void>;

    on<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    on(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;

    once<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    once(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;

    many<Event extends EventDomains<Events>>(event: Event, count: number, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    many(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;

    off(listener: EventListener): void;
    off(listener: Function): void;
    off<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): void;

    registeredEvents(): (keyof Events)[];

    listenerCount(): number;
    listenerCount<Event extends EventDomains<Events>>(event: Event): number;

    //TODO: waitFor
    //TODO: listenTo (connect emitters)
}

export class EventEmitter<Events> implements IEventEmitter<Events> {
    private readonly emitter: EmitterImplementation;
    constructor() {
        this.emitter = new EmitterImplementation({
            delimiter: '.',
            wildcard: true
        });
    }

    private createListener(event: string, listener: Function): EventListener {
        const emitter = this;
        const eventListener = function () { emitter.off(event as any, eventListener); };
        eventListener.listener = listener;
        eventListener.unregister = eventListener;
        eventListener.target = event;
        return eventListener;
    }

    emit<Event extends keyof Events>(event: Event, ...values: EventPayload<Events[Event]>): void {
        this.emitter.emit(event as any, ...values);
    }

    emitAsync<Event extends keyof Events>(event: Event, ...values: EventPayload<Events[Event]>): Promise<void> {
        return this.emitter.emitAsync(event as any, ...values).then(() => { /* omit return values */ });
    }

    on<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    on(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;
    on(event: any, listener: any): EventListener {
        if (event === "*") {
            const wrappedListener = (event: string | string[], ...values: any[]) => {
                if (Array.isArray(event)) {
                    event = event.join(".");
                }

                listener(event, ...values);
            };

            this.emitter.onAny(wrappedListener);
            return this.createListener(event, wrappedListener);
        } else {
            this.emitter.on(event, listener);
            return this.createListener(event, listener);
        }
    }

    many<Event extends EventDomains<Events>>(event: Event, count: number, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    many(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;
    many(event: any, count: any, listener?: any): EventListener {
        if (event === "*") {
            throw new Error("Any events for many listener currently not supported");
        }

        this.emitter.many(event, count, listener);
        return this.createListener(event, listener);
    }

    once<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): EventListener;
    once(event: "*", listener: <Event extends keyof Events>(event: Event, values: EventPayload<Events[Event]>) => void): EventListener;
    once(event: any, listener: any): EventListener {
        if (event === "*") {
            const emitter = this;
            const wrappedListener = (event: string | string[], ...values: any[]) => {
                if (Array.isArray(event)) {
                    event = event.join(".");
                }

                try {
                    listener(event, ...values);
                } finally {
                    emitter.off(wrappedListener);
                }
            };

            this.emitter.onAny(wrappedListener);
            return this.createListener(event, wrappedListener);
        } else {
            this.emitter.once(event, listener);
            return this.createListener(event, listener);
        }
    }

    off(listener: EventListener): void;
    off(listener: Function): void; // TODO: Does not work!
    off<Event extends EventDomains<Events>>(event: Event, listener: (...values: ListenerPayload<Events, Event>) => void): void;
    off(eventOrListener: any, listener?: any): void {
        if (eventOrListener === "*") {
            throw new Error("Using .off() with a any listener is currently not supported. Use .unregister() instead.");
        }

        this.emitter.off(eventOrListener, listener);
    }

    registeredEvents(): (keyof Events)[] {
        return this.emitter.eventNames(false) as any;
    }

    listenerCount<Event extends EventDomains<Events>>(event?: Event): number {
        return this.emitter.listenerCount(event as any);
    }
}
