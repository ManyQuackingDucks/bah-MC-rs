package com.duck.bahmod;

//import static com.duck.bahmod.BAHMod.MessageChat;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketException;

//Communication server to communicate between Rust and Java
public class Server implements Runnable{
    private static ServerSocket serverSocket;
    private static Socket clientSocket;
    private static PrintWriter out;
    private static BufferedReader in;
    private static native void rs_start();
    private static native void rs_add(String input);
    private static native void rs_del(String input);

    static {
        // This actually loads the shared object that we'll be creating.
        // The actual location of the .so or .dll may differ based on your
        // platform.
        System.loadLibrary("rslib");
    }

    public static void start(int port) throws IOException {
        serverSocket = new ServerSocket(port);
        clientSocket = serverSocket.accept();
        //debug
        System.out.println("Client Connected");
        out = new PrintWriter(clientSocket.getOutputStream(), true);
        in = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
        String inputLine;
        while ((inputLine = in.readLine()) != null) {
            //MessageChat(inputLine);
        }
        stop();
    }
    //WARNING MUST BE VALID JSON THE JSON PARSER CAN PARSE, CREATES A HARD TO DIAGNOSE ERROR
    public static void sendMessage(String command) {
        //debug
        System.out.println(command);
        out.println(command);
    }
    public static void stop() throws IOException {
        in.close();
        out.close();
        clientSocket.close();
        serverSocket.close();
    }

    public void run() {
        //debug
        System.out.println("Server started");
        while(true) {
            try {
                try {
                    start(6666);
                } catch (SocketException e) {
                    stop();
                    start(6666);
                }
            } catch (IOException e) {
                e.printStackTrace();

            }
        }
    }

}

