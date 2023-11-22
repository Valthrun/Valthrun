import * as React from "react";
import { SubscriberClient } from "../../../backend/connection";

const Context = React.createContext<SubscriberClient>(null);
export const SubscriberClientProvider = React.memo((props: {
    address: string,
    children: React.ReactNode
}) => {
    const connection = React.useMemo(() => {
        const connection = new SubscriberClient(props.address);
        return connection;
    }, [props.address]);

    return (
        <Context.Provider value={connection}>
            {props.children}
        </Context.Provider>
    );
});

export const useSubscriberClient = () => {
    const client = React.useContext(Context);
    if (!client) {
        throw new Error("no subscriber client");
    }

    return client;
}