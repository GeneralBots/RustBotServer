let items  = FIND "gb.rob", "ACTION=EMUL1"
FOR EACH item IN items  

    PRINT item.company
    let website = WEBSITE OF item.company
    PRINT website
    
    let page = GET website

    let prompt = "Build the same simulator , but for " + item.company + " using just *content about the company* from its website, so it is possible to create a good and useful emulator in the same langue as the content: " + page

    let alias = LLM "Return a single word for " + item.company + " like a token, no spaces, no special characters, no numbers, no uppercase letters."

    CREATE_SITE alias, "OpenSourceCars", prompt 

    let to = item.emailcto
    let subject = "Simulador " + alias     
    let body = "Oi, " + FIRST(item.Contact) + "! Tudo bem? Estou empolgado, pois criamos o simulador " + alias + " especificamente para vocês!"      + "\n\n Acesse o site: https://sites.pragmatismo.com.br/" + alias      + "\n\n" + "Para acessar o simulador, clique no link acima ou copie e cole no seu navegador."     + "\n\n" + "Para iniciar, clique no ícone de Play."     + "\n\n" + "Atenciosamente,\nDário Vieira\n\n"

	CREATE_DRAFT to, subject, body

    
    
NEXT item