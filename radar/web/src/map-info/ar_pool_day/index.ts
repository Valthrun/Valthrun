import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "ar_pool_day",
    displayName: "Pool Day",

    pos_x: -1088, // upper left world coordinate
    pos_y: 1600,
    scale: 2.125000,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault
            }
        }
    ]
} satisfies LoadedMap;
