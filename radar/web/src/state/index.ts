import { configureStore } from '@reduxjs/toolkit'
import { TypedUseSelectorHook, useDispatch, useSelector } from "react-redux";
import kReducerRadarSettings from "./radar-settings";
import { persistStore } from "redux-persist";
import {
    FLUSH as ReduxPersistFlush,
    REHYDRATE as ReduxPersistRehydrate,
    PAUSE as ReduxPersistPause,
    PERSIST as ReduxPersistPersist,
    PURGE as ReduxPersistPurge,
    REGISTER as ReduxPersistRegister,
} from "redux-persist";

export const appStore = configureStore({
    reducer: {
        radarSettings: kReducerRadarSettings,
    },
    middleware: getDefaultMiddleware => {

        return getDefaultMiddleware({
            serializableCheck: {
                ignoredActions: [
                    ReduxPersistFlush,
                    ReduxPersistRehydrate,
                    ReduxPersistPause,
                    ReduxPersistPersist,
                    ReduxPersistPurge,
                    ReduxPersistRegister
                ]
            }
        })
    }
});

export async function initializeAppStore() {
    await new Promise<void>(resolve => persistStore(appStore, null, resolve));
}

export type RootState = ReturnType<typeof appStore.getState>
export type AppDispatch = typeof appStore.dispatch

export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
