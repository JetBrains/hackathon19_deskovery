package com.intellij.labs.elmot.deskovery;

import com.google.gson.Gson;

import java.nio.charset.StandardCharsets;

@SuppressWarnings("WeakerAccess")
public class ControlData {
    public int x;
    public int y;
    public boolean b1;
    public boolean b2;
    public boolean b3;
    public boolean b4;
    public byte[] lastRepresentation = null;

    public void fillFromHtmlFormData(String formData) {
        final String[] valuePairs = formData.split("&");
        for (String s : valuePairs) {
            final String[] split = s.split("=");
            switch (split[0]) {
                case "x":
                    x = (int) (1000 * Double.parseDouble(split[1]));
                    break;
                case "y":
                    y = (int) (1000 * Double.parseDouble(split[1]));
                    break;
                case "b1":
                    b1 = Boolean.parseBoolean(split[1]);
                    break;
                case "b2":
                    b2 = Boolean.parseBoolean(split[1]);
                    break;
                case "b3":
                    b3 = Boolean.parseBoolean(split[1]);
                    break;
                case "b4":
                    b4 = Boolean.parseBoolean(split[1]);
                    break;
            }
        }
        lastRepresentation = null;
    }

    public byte[] robotCommand() {
        if (lastRepresentation == null) {
            lastRepresentation = new Gson().toJson(this).getBytes(StandardCharsets.UTF_8);
        }
        return lastRepresentation;
    }

}
