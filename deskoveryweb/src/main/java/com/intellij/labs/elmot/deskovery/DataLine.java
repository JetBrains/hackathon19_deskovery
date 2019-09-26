package com.intellij.labs.elmot.deskovery;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;

@SuppressWarnings("WeakerAccess")
public class DataLine {
    private static final Type ARRAY_TYPE = TypeToken.getArray(TypeToken.get(DataLine.class).getType()).getType();
    public final double x;
    public final double y;
    public final double th;
    public final double dto;
    public final boolean ps1;
    public final boolean ps2;
    public final boolean ps3;
    public final boolean ps4;

    public DataLine(double x, double y, double th, double dto, boolean ps1, boolean ps2, boolean ps3, boolean ps4) {
        this.x = x;
        this.y = y;
        this.th = th;
        this.dto = dto;
        this.ps1 = ps1;
        this.ps2 = ps2;
        this.ps3 = ps3;
        this.ps4 = ps4;
    }

    public static DataLine[] parseStream(InputStream inputStream) throws IOException {
        try (Reader reader = new InputStreamReader(inputStream, StandardCharsets.UTF_8)) {
            return (DataLine[]) new Gson().fromJson(reader, ARRAY_TYPE);
        }

    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;

        DataLine dataLine = (DataLine) o;

        if (Double.compare(dataLine.x, x) != 0) return false;
        if (Double.compare(dataLine.y, y) != 0) return false;
        if (Double.compare(dataLine.th, th) != 0) return false;
        if (Double.compare(dataLine.dto, dto) != 0) return false;
        if (ps1 != dataLine.ps1) return false;
        if (ps2 != dataLine.ps2) return false;
        if (ps3 != dataLine.ps3) return false;
        return ps4 == dataLine.ps4;
    }

    @Override
    public int hashCode() {
        int result;
        long temp;
        temp = Double.doubleToLongBits(x);
        result = (int) (temp ^ (temp >>> 32));
        temp = Double.doubleToLongBits(y);
        result = 31 * result + (int) (temp ^ (temp >>> 32));
        temp = Double.doubleToLongBits(th);
        result = 31 * result + (int) (temp ^ (temp >>> 32));
        temp = Double.doubleToLongBits(dto);
        result = 31 * result + (int) (temp ^ (temp >>> 32));
        result = 31 * result + (ps1 ? 1 : 0);
        result = 31 * result + (ps2 ? 1 : 0);
        result = 31 * result + (ps3 ? 1 : 0);
        result = 31 * result + (ps4 ? 1 : 0);
        return result;
    }
}
