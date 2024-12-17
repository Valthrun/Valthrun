import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "ar_shoots",
    displayName: "Shoots",

    pos_x: -1368, // upper left world coordinate
    pos_y: 1952,
    scale: 2.687500,

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
