const HtmlWebpackPlugin = require("html-webpack-plugin");
const webpack = require("webpack");
const path = require("path");

const isDevelopment = process.env["NODE_ENV"] === "development";
console.log(`Starting in ${isDevelopment ? "development" : "production"} mode`);

module.exports = {
    entry: "./src/index.ts",
    mode: isDevelopment ? "development" : "production",
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: "babel-loader",
                exclude: /node_modules/,
            },
            {
                test: /\.m?js/,
                resolve: {
                    fullySpecified: false,
                },
            },
            {
                test: /\.s[ac]ss$/i,
                use: ["style-loader", "css-loader", "sass-loader"],
            },
            {
                test: /\.css$/i,
                use: ["style-loader", "css-loader"],
            },
            {
                test: /\.svg$/i,
                issuer: /\.[jt]sx?$/,
                use: ["@svgr/webpack"],
            },
            {
                test: /\.(png|jpe?g|gif|jp2|webp)$/,
                loader: "file-loader",
                options: {
                    name: "assets/[contenthash].[ext]",
                },
            },
            {
                test: /\.(woff(2)?|eot|ttf|otf|)$/,
                type: "asset",
                parser: {
                    dataUrlCondition: {
                        maxSize: 8 * 1024, // 8kb
                    },
                },
                generator: {
                    filename: "assets/[hash].[ext]",
                },
            },
        ],
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js"],
    },
    output: {
        filename: "assets/web-radar.[contenthash].js",
        path: path.resolve(__dirname, "dist"),
        publicPath: "/",
        clean: true,
    },
    plugins: [
        new HtmlWebpackPlugin({
            title: "Valthrun Radar",
            favicon: path.resolve(__dirname, "src", "assets", "favicon.ico"),
        }),
        new webpack.DefinePlugin({
            "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV || "development"),
            "process.env.SERVER_URL": JSON.stringify(process.env.SERVER_URL || undefined),
        }),
    ],

    devServer: {
        historyApiFallback: true,
    },
};
