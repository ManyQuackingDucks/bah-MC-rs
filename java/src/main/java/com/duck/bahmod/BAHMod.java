package com.duck.bahmod;

import com.duck.bahmod.commands.*;
import net.minecraft.client.Minecraft;
import net.minecraft.util.ChatComponentText;
import net.minecraftforge.client.ClientCommandHandler;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.common.Mod.EventHandler;
import net.minecraftforge.fml.common.event.FMLInitializationEvent;

@Mod(modid = BAHMod.MODID, version = BAHMod.VERSION, name = BAHMod.NAME, clientSideOnly = true) // This mod is a client mod
public class BAHMod
{
    public static final String MODID = "bahmod";
    public static final String VERSION = "1.0 DEV BUILD DO NOT USE";
    public static final String NAME = "Bot's AH mod";

    @EventHandler
    public void init(FMLInitializationEvent event){
        ClientCommandHandler.instance.registerCommand(new BAHDel());
        ClientCommandHandler.instance.registerCommand(new BAHAdd());
        Server s1 = new Server();
        Thread serverThread = new Thread(s1);
        serverThread.start();
    }
    public static void MessageChat(String message){
        Minecraft.getMinecraft().thePlayer.addChatMessage(new ChatComponentText("BAH: " + message ));
        //To ensure all messages sent to the user are easily recognised.
    }
}
