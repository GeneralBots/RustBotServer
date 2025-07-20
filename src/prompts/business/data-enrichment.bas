let items  = FIND "gb.rob", "ACTION=EMUL1"
FOR EACH item IN items  
    let text = GET "https://pragmatismo.com.br" 
    PRINT item.company
NEXT item 