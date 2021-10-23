package com.duck.bahmod.commands;


import com.duck.bahmod.Server;
import net.minecraft.command.CommandBase;
import net.minecraft.command.ICommandSender;

import static com.duck.bahmod.BAHMod.MessageChat;

public class BAHDel extends CommandBase {

    @Override
    public void processCommand(ICommandSender sender, String[] params) {
        try {
            Server.del("{\"item\": \"" + params[0] + "\", \"price\": \" \", \"rarity\": \" \"}");
            MessageChat("Item Removed");
        } catch(ArrayIndexOutOfBoundsException e){
            MessageChat("§4 Not enough arguments");
        }

    }

    @Override
    public String getCommandName() {
        return "bahdel";
    }

    @Override
    public String getCommandUsage(ICommandSender sender) {
        return "command.bahdel.usage";
    }
    @Override
    public boolean canCommandSenderUseCommand(ICommandSender sender){
        return true;
    }
}
