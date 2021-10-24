package com.duck.bahmod.commands;

import com.duck.bahmod.Server;
import net.minecraft.command.CommandBase;
import net.minecraft.command.ICommandSender;

import static com.duck.bahmod.BAHMod.MessageChat;

public class BAHAdd extends CommandBase {

    @Override
    public void processCommand(ICommandSender sender, String[] params) {
        try {
            try { // Check for rarity else null params 0 and 1 are the only one required
                params[2] = params[2];
            } catch (ArrayIndexOutOfBoundsException e) {
                params[2] = "";
            }
            Server.sendClient("{\"command\": \"add\", \"item\": \"" + params[0] + "\", \"price\": \"" + params[1] + "\", \"rarity\": \"" + params[2] + "\"}"); // Command:Item Name:Price:Rarity
            MessageChat("Item Added");
        } catch(ArrayIndexOutOfBoundsException e){
            MessageChat("§4 Not enough arguments");
        }

    }

    @Override
    public String getCommandName() {
        return "bahadd";
    }

    @Override
    public String getCommandUsage(ICommandSender sender) {
        return "command.bahadd.usage";
    }
    @Override
    public boolean canCommandSenderUseCommand(ICommandSender sender){
        return true;
    }
}