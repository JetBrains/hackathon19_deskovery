package com.intellij.labs.elmot.deskovery;

import com.google.gson.Gson;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;

import java.io.*;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentLinkedQueue;

public class DeskoveryServer {
    private static final byte[] INDEX_HTML;

    static {
        try (InputStream stream =
                     DeskoveryServer.class.getResourceAsStream("/index.html");
             final ByteArrayOutputStream baos = new ByteArrayOutputStream(stream.available())
        ) {
            for (int c; (c = stream.read()) >= 0; ) {
                baos.write(c);
            }
            INDEX_HTML = baos.toByteArray();
        } catch (IOException e) {
            throw new Error(e);
        }
    }

    public static void main(String[] args) {
        InetSocketAddress inetSocketAddress = new InetSocketAddress(8000);
        try {
            switch (args.length) {
                case 0:
                    break;
                case 1:
                    final String[] split = args[0].split(":");
                    switch (split.length) {
                        case 0:
                            break;
                        case 1:
                            inetSocketAddress = new InetSocketAddress(Integer.parseInt(split[0]));
                            break;
                        case 2:
                            inetSocketAddress = new InetSocketAddress(split[0], Integer.parseInt(split[1]));
                            break;
                        default:
                            throw new IllegalArgumentException();
                    }
                    break;
                default:
                    throw new IllegalArgumentException();
            }
            new DeskoveryServer().start(inetSocketAddress);
        } catch (Throwable e) {
            System.err.println("Please specify port, host:port, or leave command line empty");
        }
    }

    private final Map<Long, Collection<DataLine>> archivedMaps = new ConcurrentHashMap<>();
    private final Collection<DataLine> activeMap = new ConcurrentLinkedQueue<>();
    private DataLine lastDataLine = null;

    private volatile ControlData lastJoystickControl = new ControlData();

    private void start(InetSocketAddress inetSocketAddress) throws IOException {
        HttpServer httpServer = HttpServer.create(inetSocketAddress, 100);
        httpServer.createContext("/", this::showIndex);
        httpServer.createContext("/poll", this::handleRobotData);
        httpServer.createContext("/control", this::handleControl);
        httpServer.createContext("/archive", this::handleArchive);
        httpServer.createContext("/map-data", this::showData);
        httpServer.start();
        System.out.println("Server started at " + httpServer.getAddress());
    }

    private void showIndex(HttpExchange exchange) throws IOException {
        if (!"/".equals(exchange.getRequestURI().getPath())) {
            exchange.sendResponseHeaders(404, -1);
            return;
        }
        exchange.sendResponseHeaders(200, INDEX_HTML.length);
        exchange.getResponseHeaders().add("Content-type", "text/html;charset=utf-8");
        try (OutputStream responseBody = exchange.getResponseBody()) {
            responseBody.write(INDEX_HTML);
        }
    }

    private void handleRobotData(HttpExchange exchange) throws IOException {
        final DataLine[] dataLines = DataLine.parseStream(new BufferedInputStream(exchange.getRequestBody()));
        for (DataLine dataLine : dataLines) {
            if (!dataLine.equals(lastDataLine)) {
                lastDataLine = dataLine;
                activeMap.add(dataLine);
            }
        }
        final byte[] bytes = lastJoystickControl.robotCommand();
        exchange.sendResponseHeaders(200, bytes.length);
        try (OutputStream out = exchange.getResponseBody()) {
            out.write(bytes);
        }
    }

    private void showData(HttpExchange exchange) throws IOException {

        exchange.getResponseHeaders().add("Content-type", "application/json;charset=utf-8");
        String query = exchange.getRequestURI().getQuery();
        Collection<DataLine> map = activeMap;

        if (query != null && !query.isEmpty()) {
            map = archivedMaps.getOrDefault(Long.parseLong(query), Collections.emptyList());
        }

        final Gson gson = new Gson();
        final byte[] bytes = gson.toJson(new DataExchange(archivedMaps.keySet(), map)).getBytes(StandardCharsets.UTF_8);
        exchange.sendResponseHeaders(200, bytes.length);
        try (OutputStream responseBody = exchange.getResponseBody()) {
            responseBody.write(bytes);
        }

    }

    private void handleControl(HttpExchange exchange) throws IOException {
        lastJoystickControl.fillFromHtmlFormData(exchange.getRequestURI().getQuery());
        exchange.sendResponseHeaders(204, -1);
    }

    private void handleArchive(HttpExchange exchange) throws IOException {
        if (activeMap.size() > 0) {
            archivedMaps.put(System.currentTimeMillis(), new ArrayList<>(activeMap));
            activeMap.clear();
        }
        exchange.getResponseHeaders().add("Location", "/");
        exchange.sendResponseHeaders(302, -1);
    }

    @SuppressWarnings("WeakerAccess")
    private static class DataExchange {
        public final Collection<Long> maps;
        public final Collection<DataLine> mapData;

        public DataExchange(Collection<Long> maps, Collection<DataLine> mapData) {
            this.maps = maps;
            this.mapData = mapData;
        }
    }
}
